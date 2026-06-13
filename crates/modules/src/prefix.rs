pub const MODULE_PREFIX: &str = "std:";

pub const MODULE_ENV: &str = "std:env";
pub const MODULE_FFI: &str = "std:ffi";
pub const MODULE_FS: &str = "std:fs";
pub const MODULE_HTTP: &str = "std:http";
pub const MODULE_PATH: &str = "std:path";
pub const MODULE_PROCESS: &str = "std:process";
pub const MODULE_SHELL: &str = "std:shell";
pub const MODULE_SOCKET: &str = "std:socket";
pub const MODULE_SPAWN: &str = "std:spawn";
pub const MODULE_TCP: &str = "std:tcp";
pub const MODULE_TEST: &str = "std:test";
pub const MODULE_TRACING: &str = "std:tracing";
pub const MODULE_UDP: &str = "std:udp";
pub const MODULE_WASM: &str = "std:wasm";

/// Public `std:*` modules that are allowed to import internal `std:raw:*` native modules.
/// This list is the source of truth for resolver access control.
pub const PUBLIC_STD_MODULES: &[&str] = &[
    MODULE_ENV,
    MODULE_FFI,
    MODULE_FS,
    MODULE_HTTP,
    MODULE_PATH,
    MODULE_PROCESS,
    MODULE_SHELL,
    MODULE_SOCKET,
    MODULE_SPAWN,
    MODULE_TCP,
    MODULE_TEST,
    MODULE_UDP,
    MODULE_WASM,
];
