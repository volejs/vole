use rquickjs::{Ctx, Error, Module};
use vole_module::NativeModuleProvider;

use crate::raw_bindings::{
    console, crypto, env, ffi, file, fs, http_server, path, process, socket, tcp, tracing, udp,
    wasm,
};

pub struct RuntimeNativeModuleProvider;

impl NativeModuleProvider for RuntimeNativeModuleProvider {
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
        match name {
            "std:raw:console" => Some(Module::declare_def::<console::ConsoleRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:crypto" => Some(Module::declare_def::<crypto::CryptoRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:env" => Some(Module::declare_def::<env::EnvRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:ffi" => Some(Module::declare_def::<ffi::FfiRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:file" => Some(Module::declare_def::<file::FileRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:fs" => Some(Module::declare_def::<fs::FsRawModule, _>(ctx.clone(), name)),
            "std:raw:http" => Some(Module::declare_def::<http_server::HttpRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:path" => Some(Module::declare_def::<path::PathRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:process" => Some(Module::declare_def::<process::ProcessRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:socket" => Some(Module::declare_def::<socket::SocketRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:tcp" => Some(Module::declare_def::<tcp::TcpRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:tracing" => Some(Module::declare_def::<tracing::TracingRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:udp" => Some(Module::declare_def::<udp::UdpRawModule, _>(
                ctx.clone(),
                name,
            )),
            "std:raw:wasm" => Some(Module::declare_def::<wasm::WasmRawModule, _>(
                ctx.clone(),
                name,
            )),
            _ => None,
        }
    }
}

/// Register all Rust-native `std:*` modules.
pub fn provider() -> RuntimeNativeModuleProvider {
    RuntimeNativeModuleProvider
}
