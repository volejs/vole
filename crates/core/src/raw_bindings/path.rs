use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Ctx, Object, Result,
};

fn join(parts: Vec<String>) -> String {
    let path: std::path::PathBuf = parts.iter().collect();
    path.to_string_lossy().to_string()
}

fn resolve(parts: Vec<String>) -> String {
    let mut result = std::path::PathBuf::new();
    for part in parts {
        let p = std::path::Path::new(&part);
        if p.is_absolute() {
            result = p.to_path_buf();
        } else {
            result.push(p);
        }
    }
    result.to_string_lossy().to_string()
}

fn dirname(path: String) -> String {
    std::path::Path::new(&path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_string_lossy()
        .to_string()
}

fn basename(path: String, ext: Option<String>) -> String {
    let p = std::path::Path::new(&path);
    let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
    if let Some(ext) = ext {
        if name.ends_with(&ext) {
            return name[..name.len() - ext.len()].to_string();
        }
    }
    name
}

fn extname(path: String) -> String {
    std::path::Path::new(&path)
        .extension()
        .map(|s| {
            let mut e = String::from(".");
            e.push_str(&s.to_string_lossy());
            e
        })
        .unwrap_or_default()
}

fn is_absolute(path: String) -> bool {
    std::path::Path::new(&path).is_absolute()
}

fn relative(from: String, to: String) -> String {
    let from_path = std::path::Path::new(&from);
    let to_path = std::path::Path::new(&to);
    let result = from_path
        .strip_prefix(to_path)
        .unwrap_or(to_path.strip_prefix(from_path).unwrap_or(to_path));
    result.to_string_lossy().to_string()
}

fn parse<'js>(ctx: Ctx<'js>, path: String) -> Result<Object<'js>> {
    let p = std::path::Path::new(&path);
    let obj = Object::new(ctx)?;
    obj.set("root", p.parent().map(|_| String::new()).unwrap_or_default())?;
    obj.set(
        "dir",
        p.parent()
            .unwrap_or(std::path::Path::new(""))
            .to_string_lossy()
            .to_string(),
    )?;
    obj.set(
        "base",
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    )?;
    let name = p
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    obj.set("name", name)?;
    let ext = p
        .extension()
        .map(|s| {
            let mut e = String::from(".");
            e.push_str(&s.to_string_lossy());
            e
        })
        .unwrap_or_default();
    obj.set("ext", ext)?;
    Ok(obj)
}

fn format(path_obj: Object) -> Result<String> {
    let dir: Option<String> = path_obj.get("dir").ok();
    let base: Option<String> = path_obj.get("base").ok();
    let name: Option<String> = path_obj.get("name").ok();
    let ext: Option<String> = path_obj.get("ext").ok();
    let root: Option<String> = path_obj.get("root").ok();
    let sep = std::path::MAIN_SEPARATOR;

    if let Some(base) = base {
        if let Some(dir) = dir {
            return Ok(format!("{}{}{}", dir, sep, base));
        }
        return Ok(base);
    }

    let name = name.unwrap_or_default();
    let ext = ext.unwrap_or_default();
    let filename = format!("{}{}", name, ext);

    if let Some(dir) = dir {
        Ok(format!("{}{}{}", dir, sep, filename))
    } else if let Some(root) = root {
        Ok(format!("{}{}", root, filename))
    } else {
        Ok(filename)
    }
}

/// Native module exposing the low-level path operations used by the `std:path` JS shim.
/// The functions here take the same shapes as the legacy `globalThis.__path_*` helpers
/// (e.g. `__path_join` receives an array of strings), so the JS shim can simply import
/// and re-export them without touching `globalThis`.
pub struct PathRawModule;

impl ModuleDef for PathRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__path_join")?;
        decl.declare("__path_resolve")?;
        decl.declare("__path_dirname")?;
        decl.declare("__path_basename")?;
        decl.declare("__path_extname")?;
        decl.declare("__path_isAbsolute")?;
        decl.declare("__path_relative")?;
        decl.declare("__path_parse")?;
        decl.declare("__path_format")?;
        decl.declare("__path_sep")?;
        decl.declare("__path_delimiter")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__path_join", rquickjs::Function::new(ctx.clone(), join))?;
        exports.export("__path_resolve", rquickjs::Function::new(ctx.clone(), resolve))?;
        exports.export("__path_dirname", rquickjs::Function::new(ctx.clone(), dirname))?;
        exports.export("__path_basename", rquickjs::Function::new(ctx.clone(), basename))?;
        exports.export("__path_extname", rquickjs::Function::new(ctx.clone(), extname))?;
        exports.export("__path_isAbsolute", rquickjs::Function::new(ctx.clone(), is_absolute))?;
        exports.export("__path_relative", rquickjs::Function::new(ctx.clone(), relative))?;
        exports.export("__path_parse", rquickjs::Function::new(ctx.clone(), parse))?;
        exports.export("__path_format", rquickjs::Function::new(ctx.clone(), format))?;
        exports.export("__path_sep", std::path::MAIN_SEPARATOR.to_string())?;
        exports.export("__path_delimiter", ":")?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `std:path`.
/// All path helpers are provided through the native module `std:raw:path` and consumed by
/// the `std:path` JS shim via ES module imports.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}
