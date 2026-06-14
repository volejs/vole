use rquickjs::{Ctx, Error, Module};
use vole_module::{vole_raw_mod, NativeModuleProvider};

use vole_raw_env::EnvRawModule as env;
use vole_raw_ffi::FfiRawModule as ffi;
use vole_raw_fs::FsRawModule as fs;
use vole_raw_net::{
    HttpRawModule as http, SocketRawModule as socket, TcpRawModule as tcp, UdpRawModule as udp,
};
use vole_raw_path::PathRawModule as path;
use vole_raw_process::ProcessRawModule as process;
use vole_raw_wasm::WasmRawModule as wasm;

pub struct RuntimeNativeModuleProvider;

impl NativeModuleProvider for RuntimeNativeModuleProvider {
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
        match name {
            vole_raw_mod!(env) => Some(Module::declare_def::<env, _>(ctx.clone(), name)),
            vole_raw_mod!(ffi) => Some(Module::declare_def::<ffi, _>(ctx.clone(), name)),
            vole_raw_mod!(fs) => Some(Module::declare_def::<fs, _>(ctx.clone(), name)),
            vole_raw_mod!("net/http") => Some(Module::declare_def::<http, _>(ctx.clone(), name)),
            vole_raw_mod!(path) => Some(Module::declare_def::<path, _>(ctx.clone(), name)),
            vole_raw_mod!(process) => Some(Module::declare_def::<process, _>(ctx.clone(), name)),
            vole_raw_mod!("net/socket") => {
                Some(Module::declare_def::<socket, _>(ctx.clone(), name))
            }
            vole_raw_mod!("net/tcp") => Some(Module::declare_def::<tcp, _>(ctx.clone(), name)),
            vole_raw_mod!("net/udp") => Some(Module::declare_def::<udp, _>(ctx.clone(), name)),
            vole_raw_mod!(wasm) => Some(Module::declare_def::<wasm, _>(ctx.clone(), name)),
            _ => None,
        }
    }
}

/// Register all Rust-native `vole:*` modules.
pub fn provider() -> RuntimeNativeModuleProvider {
    RuntimeNativeModuleProvider
}
