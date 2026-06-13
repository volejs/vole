/// Produce a built-in module specifier string like `vole:env` or `vole:net/http`.
#[macro_export]
macro_rules! vole_mod {
    ($name:ident) => {
        concat!("vole:", stringify!($name))
    };
    ($path:literal) => {
        concat!("vole:", $path)
    };
}

/// Produce an internal raw module specifier string like `vole:raw:fs`
/// or `vole:raw:net/http`.
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
/// `vole_js_shim!("net/http")` expands to `include_str!("js_shims/net/http.js")`.
/// The path is resolved relative to the file where the macro is invoked,
/// so it should be called from `loader.rs`.
#[macro_export]
macro_rules! vole_js_shim {
    ($path:literal) => {
        include_str!(concat!("js_shims/", $path, ".js"))
    };
}

/// Public `vole:*` modules that are allowed to import internal `vole:raw:*`
/// native modules. This list is the source of truth for resolver access control.
pub const PUBLIC_VOLE_MODULES: &[&str] = &[
    vole_mod!(env),
    vole_mod!(ffi),
    vole_mod!(fs),
    vole_mod!(path),
    vole_mod!(process),
    vole_mod!(shell),
    vole_mod!(spawn),
    vole_mod!(test),
    vole_mod!(wasm),
    // Network namespace
    vole_mod!("net/http"),
    vole_mod!("net/socket"),
    vole_mod!("net/tcp"),
    vole_mod!("net/udp"),
];
