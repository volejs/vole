use rquickjs::{loader::Loader, module::ModuleDef, Ctx, Error, Module};
use std::collections::HashMap;

use crate::{vole_js_shim, vole_mod};

/// Allows the embedding runtime to provide Rust-native modules for `vole:*` specifiers.
pub trait NativeModuleProvider: Send + Sync {
    /// Attempt to load a native module by name. Returns `None` to fall back to JS shim loading.
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>>;
}

pub struct ModuleLoader {
    vole_modules: HashMap<String, String>,
    native_providers: Vec<Box<dyn NativeModuleProvider>>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        let mut vole_modules = HashMap::new();

        macro_rules! insert_shim {
            ($map:ident, $path:literal) => {
                $map.insert(vole_mod!($path).to_string(), vole_js_shim!($path));
            };
        }

        insert_shim!(vole_modules, "env");
        insert_shim!(vole_modules, "ffi");
        insert_shim!(vole_modules, "fs");
        insert_shim!(vole_modules, "path");
        insert_shim!(vole_modules, "process");
        insert_shim!(vole_modules, "shell");
        insert_shim!(vole_modules, "spawn");
        insert_shim!(vole_modules, "test");
        insert_shim!(vole_modules, "wasm");
        // Network namespace
        insert_shim!(vole_modules, "net/http");
        insert_shim!(vole_modules, "net/socket");
        insert_shim!(vole_modules, "net/tcp");
        insert_shim!(vole_modules, "net/udp");

        ModuleLoader {
            vole_modules,
            native_providers: Vec::new(),
        }
    }

    /// Register a provider for Rust-native modules (e.g. `vole:path`).
    /// Providers are consulted in registration order before falling back to bundled JS shims.
    pub fn add_native_provider(&mut self, provider: Box<dyn NativeModuleProvider>) {
        self.native_providers.push(provider);
    }

    /// Convenience helper to register a single native module by specifier.
    pub fn register_native_module<D: ModuleDef + Send + Sync + 'static>(
        &mut self,
        name: impl Into<String>,
    ) {
        let name = name.into();
        struct DefNativeModuleProvider<D> {
            name: String,
            _marker: std::marker::PhantomData<D>,
        }
        impl<D: ModuleDef + Send + Sync + 'static> NativeModuleProvider for DefNativeModuleProvider<D> {
            fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
                if name == self.name {
                    Some(Module::declare_def::<D, _>(ctx.clone(), name))
                } else {
                    None
                }
            }
        }
        let provider = DefNativeModuleProvider::<D> {
            name,
            _marker: std::marker::PhantomData,
        };
        self.native_providers.push(Box::new(provider));
    }

    fn is_vole_module(&self, name: &str) -> bool {
        name.starts_with("vole:")
    }

    fn load_vole_module<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>, Error> {
        let source = self.vole_modules.get(name).ok_or(Error::Exception)?;
        Module::declare(ctx.clone(), name, source.as_bytes().to_vec())
    }

    fn load_file<'js>(
        &self,
        ctx: &Ctx<'js>,
        name: &str,
        format: Option<String>,
    ) -> Result<Module<'js>, Error> {
        let source = std::fs::read_to_string(name).map_err(|_| Error::Exception)?;

        let module_source = if let Some(fmt) = format {
            transform_config(&source, &fmt).map_err(|_| Error::Exception)?
        } else if is_typescript_file(name) {
            transpile_typescript(&source, name).map_err(|_| Error::Exception)?
        } else {
            source.into_bytes()
        };

        Module::declare(ctx.clone(), name, module_source)
    }
}

impl Loader for ModuleLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        attrs: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> Result<Module<'js>, Error> {
        if self.is_vole_module(name) {
            for provider in &self.native_providers {
                if let Some(result) = provider.load(ctx, name) {
                    return result;
                }
            }
            self.load_vole_module(ctx, name)
        } else {
            let format = attrs
                .and_then(|a| a.get_type().ok().flatten())
                .or_else(|| infer_format_from_path(name));
            self.load_file(ctx, name, format)
        }
    }
}

fn is_typescript_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".ts")
        || path_lower.ends_with(".mts")
        || path_lower.ends_with(".cts")
        || path_lower.ends_with(".tsx")
}

pub fn transpile_typescript(
    source: &str,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use oxc_allocator::Allocator;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use oxc_transformer::{TransformOptions, Transformer};

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path)?;
    let ret = Parser::new(&allocator, source, source_type).parse();

    if !ret.errors.is_empty() {
        return Err("TypeScript parse error".into());
    }

    let mut program = ret.program;

    // Build semantic analysis for scoping info required by transformer
    let semantic_ret = SemanticBuilder::new().build(&program);
    let scoping = semantic_ret.semantic.into_scoping();

    // Strip TypeScript annotations (and any other applicable transforms)
    let transform_options = TransformOptions::default();
    let transformer = Transformer::new(&allocator, std::path::Path::new(path), &transform_options);
    transformer.build_with_scoping(scoping, &mut program);

    let codegen = Codegen::new();
    let result = codegen.build(&program);
    Ok(result.code.into_bytes())
}

fn infer_format_from_path(path: &str) -> Option<String> {
    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".json") {
        Some("json".to_string())
    } else if path_lower.ends_with(".toml") {
        Some("toml".to_string())
    } else if path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
        Some("yaml".to_string())
    } else if path_lower.ends_with(".ini") {
        Some("ini".to_string())
    } else if path_lower.ends_with(".jsonc") {
        Some("jsonc".to_string())
    } else if path_lower.ends_with(".json5") {
        Some("json5".to_string())
    } else {
        None
    }
}

fn transform_config(source: &str, format: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let value: serde_json::Value = match format {
        "json" => serde_json::from_str(source)?,
        "toml" => {
            let toml_value: toml::Value = source.parse()?;
            toml_value_to_json(toml_value)
        }
        "yaml" | "yml" => serde_yaml::from_str(source)?,
        "jsonc" => {
            let stripped = strip_jsonc_comments(source);
            serde_json::from_str(&stripped)?
        }
        "json5" => json5::from_str(source)?,
        "ini" => parse_ini(source),
        _ => serde_json::from_str(source)?,
    };

    let json_str = serde_json::to_string(&value)?;
    let module_source = format!("export default {};\n", json_str);
    Ok(module_source.into_bytes())
}

fn toml_value_to_json(value: toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => {
            serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(d) => serde_json::Value::String(d.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, v) in table {
                map.insert(k, toml_value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    chars.next();
                    while chars.next() != Some('\n') {}
                    result.push('\n');
                }
                Some(&'*') => {
                    chars.next();
                    while let Some(ch) = chars.next() {
                        if ch == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => result.push(c),
            }
        } else if c == '"' {
            result.push(c);
            while let Some(ch) = chars.next() {
                result.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        result.push(next);
                    }
                } else if ch == '"' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn parse_ini(content: &str) -> serde_json::Value {
    let mut root = serde_json::Map::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            if !root.contains_key(&current_section) {
                root.insert(
                    current_section.clone(),
                    serde_json::Value::Object(serde_json::Map::new()),
                );
            }
        } else if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            let value = parse_ini_value(value);

            if current_section.is_empty() {
                root.insert(key.to_string(), value);
            } else if let Some(serde_json::Value::Object(section)) = root.get_mut(&current_section)
            {
                section.insert(key.to_string(), value);
            }
        }
    }

    serde_json::Value::Object(root)
}

fn parse_ini_value(value: &str) -> serde_json::Value {
    if value.eq_ignore_ascii_case("true") {
        serde_json::Value::Bool(true)
    } else if value.eq_ignore_ascii_case("false") {
        serde_json::Value::Bool(false)
    } else if let Ok(n) = value.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(n) = value.parse::<f64>() {
        serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
    } else {
        serde_json::Value::String(value.to_string())
    }
}
