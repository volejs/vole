use rquickjs::{Ctx, Error, Module};
use vole_module::{vole_raw_mod, NativeModuleProvider};

use vole_raw_dns::DnsRawModule as dns;
use vole_raw_env::EnvRawModule as env;
use vole_raw_exec::OsExecRawModule as exec;
use vole_raw_ffi::FfiRawModule as ffi;
use vole_raw_fs::FsRawModule as fs;
use vole_raw_http::HttpRawModule as http;
use vole_raw_os::OsRawModule as os;
use vole_raw_path::PathRawModule as path;
use vole_raw_signal::OsSignalRawModule as signal;
use vole_raw_socket::SocketRawModule as socket;
use vole_raw_tcp::TcpRawModule as tcp;
use vole_raw_udp::UdpRawModule as udp;
use vole_raw_until::UntilRawModule as until;
use vole_raw_wasm::WasmRawModule as wasm;

pub struct RuntimeNativeModuleProvider;

impl NativeModuleProvider for RuntimeNativeModuleProvider {
    fn load<'js>(&self, ctx: &Ctx<'js>, name: &str) -> Option<Result<Module<'js>, Error>> {
        match name {
            vole_raw_mod!(dns) => Some(Module::declare_def::<dns, _>(ctx.clone(), name)),
            vole_raw_mod!(env) => Some(Module::declare_def::<env, _>(ctx.clone(), name)),
            vole_raw_mod!(exec) => Some(Module::declare_def::<exec, _>(ctx.clone(), name)),
            vole_raw_mod!(ffi) => Some(Module::declare_def::<ffi, _>(ctx.clone(), name)),
            vole_raw_mod!(fs) => Some(Module::declare_def::<fs, _>(ctx.clone(), name)),
            vole_raw_mod!(http) => Some(Module::declare_def::<http, _>(ctx.clone(), name)),
            vole_raw_mod!(os) => Some(Module::declare_def::<os, _>(ctx.clone(), name)),
            vole_raw_mod!(path) => Some(Module::declare_def::<path, _>(ctx.clone(), name)),
            vole_raw_mod!(signal) => Some(Module::declare_def::<signal, _>(ctx.clone(), name)),
            vole_raw_mod!(socket) => Some(Module::declare_def::<socket, _>(ctx.clone(), name)),
            vole_raw_mod!(tcp) => Some(Module::declare_def::<tcp, _>(ctx.clone(), name)),
            vole_raw_mod!(udp) => Some(Module::declare_def::<udp, _>(ctx.clone(), name)),
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
