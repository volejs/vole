use rquickjs::{Ctx, Error, loader::Resolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::PUBLIC_VOLE_MODULES;

pub struct ModuleResolver {
    base_dir: PathBuf,
    import_map: HashMap<String, String>,
}

impl ModuleResolver {
    pub fn new(base_dir: PathBuf) -> Self {
        let import_map = Self::load_import_map(&base_dir);

        ModuleResolver {
            base_dir,
            import_map,
        }
    }

    fn load_import_map(base_dir: &Path) -> HashMap<String, String> {
        let path = base_dir.join("importmap.json");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return HashMap::new(),
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return HashMap::new(),
        };
        let imports = json.get("imports").and_then(|v| v.as_object());
        let mut map = HashMap::new();
        if let Some(imports) = imports {
            for (key, value) in imports {
                if let Some(target) = value.as_str() {
                    map.insert(key.to_string(), target.to_string());
                }
            }
        }
        map
    }

    fn resolve_relative(&self, base: &str, name: &str) -> Result<String, Error> {
        let base_path = Path::new(base);
        let base_dir = if base_path.is_file() {
            base_path.parent().unwrap_or(Path::new("."))
        } else {
            base_path
        };

        let resolved = base_dir.join(name);
        let resolved = resolved.canonicalize().map_err(|_e| Error::Exception)?;

        Ok(resolved.to_string_lossy().to_string())
    }

    fn is_relative(&self, name: &str) -> bool {
        name.starts_with("./") || name.starts_with("../")
    }

    fn is_vole_module(&self, name: &str) -> bool {
        name.starts_with("vole:")
    }

    fn is_internal_raw_module(&self, name: &str) -> bool {
        name.starts_with("vole:raw:")
    }

    fn is_public_vole_module(&self, name: &str) -> bool {
        PUBLIC_VOLE_MODULES.contains(&name)
    }
}

impl Resolver for ModuleResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attrs: Option<rquickjs::loader::ImportAttributes<'js>>,
    ) -> Result<String, Error> {
        // vole: modules (public vole:xxx and internal raw vole:raw:xxx)
        if self.is_vole_module(name) {
            // Internal raw modules are only importable by public vole:* modules.
            if self.is_internal_raw_module(name) && !self.is_public_vole_module(base) {
                return Err(Error::Exception);
            }
            return Ok(name.to_string());
        }

        // Relative paths
        if self.is_relative(name) {
            return self.resolve_relative(base, name);
        }

        // Absolute paths
        if Path::new(name).is_absolute() {
            return Ok(name.to_string());
        }

        // Import map resolution
        if let Some(mapped) = self.resolve_import_map(name) {
            let mapped_path = Path::new(&mapped);
            if mapped_path.is_absolute() {
                // Resolve absolute mapped path relative to base_dir
                let resolved = self
                    .base_dir
                    .join(mapped_path.strip_prefix("/").unwrap_or(mapped_path));
                if let Ok(canonical) = resolved.canonicalize() {
                    return Ok(canonical.to_string_lossy().to_string());
                }
                return Ok(resolved.to_string_lossy().to_string());
            }
            return Ok(mapped);
        }

        // Unknown module
        Err(Error::Exception)
    }
}

impl ModuleResolver {
    fn resolve_import_map(&self, name: &str) -> Option<String> {
        // Exact match
        if let Some(target) = self.import_map.get(name) {
            return Some(target.clone());
        }

        // Prefix match (longest prefix wins)
        let mut best_match: Option<(&str, &str)> = None;
        for (key, target) in &self.import_map {
            if key.ends_with('/') && name.starts_with(key) {
                if best_match.map_or(true, |(best_key, _)| key.len() > best_key.len()) {
                    best_match = Some((key, target));
                }
            }
        }

        if let Some((key, target)) = best_match {
            let suffix = &name[key.len()..];
            let mapped = if target.ends_with('/') {
                format!("{}{}", target, suffix)
            } else {
                format!("{}/{}", target, suffix)
            };
            return Some(mapped);
        }

        None
    }
}
