use rquickjs::{Ctx, Error, Module};
use vole_module::NativeModuleProvider;

use vole_raw_env::EnvRawModule as env;
use vole_raw_ffi::FfiRawModule as ffi;
use vole_raw_fs::FsRawModule as fs;
use vole_raw_http_server::HttpRawModule as http_server;
use vole_raw_path::PathRawModule as path;
use vole_raw_process::ProcessRawModule as process;
use vole_raw_socket::SocketRawModule as socket;
use vole_raw_tcp::TcpRawModule as tcp;
use vole_raw_udp::UdpRawModule as udp;
use vole_raw_wasm::WasmRawModule as wasm;

pub struct RuntimeNativeModuleProvider;

impl NativeModuleProvider for RuntimeNativeModuleProvider {
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
        match name {
            "std:raw:env" => Some(Module::declare_def::<env, _>(ctx.clone(), name)),
            "std:raw:ffi" => Some(Module::declare_def::<ffi, _>(ctx.clone(), name)),
            "std:raw:fs" => Some(Module::declare_def::<fs, _>(ctx.clone(), name)),
            "std:raw:http" => Some(Module::declare_def::<http_server, _>(ctx.clone(), name)),
            "std:raw:path" => Some(Module::declare_def::<path, _>(ctx.clone(), name)),
            "std:raw:process" => Some(Module::declare_def::<process, _>(ctx.clone(), name)),
            "std:raw:socket" => Some(Module::declare_def::<socket, _>(ctx.clone(), name)),
            "std:raw:tcp" => Some(Module::declare_def::<tcp, _>(ctx.clone(), name)),
            "std:raw:udp" => Some(Module::declare_def::<udp, _>(ctx.clone(), name)),
            "std:raw:wasm" => Some(Module::declare_def::<wasm, _>(ctx.clone(), name)),
            _ => None,
        }
    }
}

/// Register all Rust-native `std:*` modules.
pub fn provider() -> RuntimeNativeModuleProvider {
    RuntimeNativeModuleProvider
}
