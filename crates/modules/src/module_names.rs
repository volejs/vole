/// Produce a built-in module specifier string like `vole:os` or `vole:http`.
#[macro_export]
macro_rules! vole_mod {
    ($name:ident) => {
        concat!("vole:", stringify!($name))
    };
    ($path:literal) => {
        concat!("vole:", $path)
    };
}

/// Produce an internal raw module specifier string like `vole:raw:env`
/// or `vole:raw:http`.
#[macro_export]
macro_rules! vole_raw_mod {
    ($name:ident) => {
        concat!("vole:raw:", stringify!($name))
    };
    ($path:literal) => {
        concat!("vole:raw:", $path)
    };
}

/// Include a bundled JS shim source by its logical module path.
///
/// `vole_js_shim!("http")` resolves to the content of
/// `crates/modules/src/js_shims/http.js` via the embedded asset store.
/// It can be called from anywhere because the path is anchored to the
/// `vole-module` crate root.
#[macro_export]
macro_rules! vole_js_shim {
    ($path:literal) => {
        $crate::js_shims_embed::get_shim($path).unwrap()
    };
}

/// Public `vole:*` modules that are allowed to import internal `vole:raw:*`
/// native modules. This list is the source of truth for resolver access control.
pub const PUBLIC_VOLE_MODULES: &[&str] = &[
    vole_mod!(ffi),
    vole_mod!(path),
    // OS-like runtime modules.
    vole_mod!("os"),
    vole_mod!("env"),
    vole_mod!("fs"),
    vole_mod!("exec"),
    vole_mod!("dns"),
    vole_mod!("signal"),
    vole_mod!("tls"),
    // Network modules.
    vole_mod!("http"),
    vole_mod!("socket"),
    vole_mod!("tcp"),
    vole_mod!("udp"),
    vole_mod!(test),
    vole_mod!(until),
    vole_mod!(wasm),
];
