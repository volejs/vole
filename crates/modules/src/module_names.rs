/// Produce a built-in module specifier string like `vole:env`.
#[macro_export]
macro_rules! vole_mod {
    ($name:ident) => {
        concat!("vole:", stringify!($name))
    };
}

/// Produce an internal raw module specifier string like `vole:raw:fs`.
#[macro_export]
macro_rules! vole_raw_mod {
    ($name:ident) => {
        concat!("vole:raw:", stringify!($name))
    };
}

/// Public `vole:*` modules that are allowed to import internal `vole:raw:*`
/// native modules. This list is the source of truth for resolver access control.
pub const PUBLIC_VOLE_MODULES: &[&str] = &[
    vole_mod!(env),
    vole_mod!(ffi),
    vole_mod!(fs),
    vole_mod!(http),
    vole_mod!(path),
    vole_mod!(process),
    vole_mod!(shell),
    vole_mod!(socket),
    vole_mod!(spawn),
    vole_mod!(tcp),
    vole_mod!(test),
    vole_mod!(udp),
    vole_mod!(wasm),
];
