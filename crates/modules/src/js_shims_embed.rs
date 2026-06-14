use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/js_shims/"]
pub struct JsShims;

/// Get the source of a bundled JS shim by its logical path (without the
/// `vole:` prefix and without the `.js` extension).
pub fn get_shim(path: &str) -> Option<String> {
    let key = format!("{}.js", path);
    JsShims::get(&key).map(|file| String::from_utf8_lossy(file.data.as_ref()).into_owned())
}
