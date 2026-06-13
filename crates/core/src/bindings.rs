use rquickjs::AsyncContext;

use crate::extensions::ExtensionRegistry;

/// 构建包含所有 Phase 0 内置扩展的注册表。
pub fn registry() -> ExtensionRegistry {
    ExtensionRegistry::new()
        // global_bindings: Web APIs exposed directly on globalThis
        .with(vole_global_timers::init)
        .with(vole_global_performance::init)
        .with(vole_global_base64::init)
        .with(vole_global_dom_exception::init)
        .with(vole_global_report_error::init)
        .with(vole_global_encoding::init)
        .with(vole_global_url::init)
        .with(vole_global_blob::init)
        .with(vole_global_crypto::init)
        .with(vole_global_structured_clone::init)
        .with(vole_global_events::init)
        .with(vole_global_fetch::init)
        .with(vole_global_console::init)
        .with(vole_global_streams::init)
        .with(vole_global_compression::init)
        // raw_bindings: low-level Rust backends exposed via std:raw:*
        .with(vole_raw_env::init)
        .with(vole_raw_ffi::init)
        .with(vole_raw_process::init)
        .with(vole_raw_path::init)
        .with(vole_raw_http_server::init)
        .with(vole_raw_tcp::init)
        .with(vole_raw_udp::init)
        .with(vole_raw_socket::init)
        .with(vole_raw_wasm::init)
}

/// Initialize all bindings.
///
/// This is the single entry point for VM initialization. It:
/// 1. Registers global Web APIs and std: module low-level functions
/// 2. Asynchronously loads Web API shims that depend on std:raw:* native modules
/// 3. Initializes the timer task registry
pub async fn init(ctx: &AsyncContext) -> Result<(), rquickjs::Error> {
    ctx.with(|ctx| {
        registry().init(&ctx)?;
        Ok::<_, rquickjs::Error>(())
    })
    .await?;

    init_async(ctx).await?;

    vole_global_timers::init_registry();

    Ok(())
}

/// Asynchronously initialize bindings that depend on Rust-native raw modules.
///
/// This must be called after `registry().init()` so that the underlying
/// Web APIs (ReadableStream/TransformStream/etc.) are already present on
/// globalThis before the shims run.
pub async fn init_async(ctx: &AsyncContext) -> Result<(), rquickjs::Error> {
    ctx.async_with(async |ctx| {
        let wasm = rquickjs::Module::evaluate(
            ctx.clone(),
            "std:wasm",
            include_str!("../../modules/src/js_shims/wasm.js"),
        )?;
        wasm.finish::<()>()?;

        Ok::<_, rquickjs::Error>(())
    })
    .await
}
