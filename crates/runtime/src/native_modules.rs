use rquickjs::{Ctx, Error, Module};
use vole_module::{NativeModuleProvider, vole_raw_mod};

use vole_raw_ffi::FfiRawModule as ffi;
use vole_raw_net::{
    HttpRawModule as http, SocketRawModule as socket, TcpRawModule as tcp, UdpRawModule as udp,
};
use vole_raw_os::{OsExecRawModule as os_exec, OsFsRawModule as os_fs, OsRawModule as os,
    OsEnvRawModule as os_env};
use vole_raw_path::PathRawModule as path;
use vole_raw_until::UntilRawModule as until;
use vole_raw_wasm::WasmRawModule as wasm;

pub struct RuntimeNativeModuleProvider;

impl NativeModuleProvider for RuntimeNativeModuleProvider {
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
        match name {
            vole_raw_mod!(ffi) => Some(Module::declare_def::<ffi, _>(ctx.clone(), name)),
            vole_raw_mod!("os") => Some(Module::declare_def::<os, _>(ctx.clone(), name)),
            vole_raw_mod!("os/env") => Some(Module::declare_def::<os_env, _>(ctx.clone(), name)),
            vole_raw_mod!("os/fs") => Some(Module::declare_def::<os_fs, _>(ctx.clone(), name)),
            vole_raw_mod!("os/exec") => Some(Module::declare_def::<os_exec, _>(ctx.clone(), name)),
            vole_raw_mod!("net/http") => Some(Module::declare_def::<http, _>(ctx.clone(), name)),
            vole_raw_mod!(path) => Some(Module::declare_def::<path, _>(ctx.clone(), name)),
            vole_raw_mod!("net/socket") => {
                Some(Module::declare_def::<socket, _>(ctx.clone(), name))
            }
            vole_raw_mod!("net/tcp") => Some(Module::declare_def::<tcp, _>(ctx.clone(), name)),
            vole_raw_mod!("net/udp") => Some(Module::declare_def::<udp, _>(ctx.clone(), name)),
            vole_raw_mod!(until) => Some(Module::declare_def::<until, _>(ctx.clone(), name)),
            vole_raw_mod!(wasm) => Some(Module::declare_def::<wasm, _>(ctx.clone(), name)),
            _ => None,
        }
    }
}

/// Register all Rust-native `vole:*` modules.
pub fn provider() -> RuntimeNativeModuleProvider {
    RuntimeNativeModuleProvider
}
