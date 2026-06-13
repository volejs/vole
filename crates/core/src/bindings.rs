use rquickjs::AsyncContext;

use crate::extensions::ExtensionRegistry;

/// 构建包含所有 Phase 0 内置扩展的注册表。
pub fn registry() -> ExtensionRegistry {
    ExtensionRegistry::new()
        // global_bindings: Web APIs exposed directly on globalThis
        .with(crate::global_bindings::timers::init)
        .with(crate::global_bindings::performance::init)
        .with(crate::global_bindings::base64::init)
        .with(crate::global_bindings::dom_exception::init)
        .with(crate::global_bindings::report_error::init)
        .with(crate::global_bindings::encoding::init)
        .with(crate::global_bindings::url::init)
        .with(crate::global_bindings::blob::init)
        .with(crate::global_bindings::crypto::init)
        .with(crate::global_bindings::structured_clone::init)
        .with(crate::global_bindings::events::init)
        .with(crate::global_bindings::fetch::init)
        // raw_bindings: low-level Rust backends exposed via std:raw:*
        .with(crate::global_bindings::console::init)
        .with(crate::raw_bindings::env::init)
        .with(crate::raw_bindings::ffi::init)
        .with(crate::raw_bindings::process::init)
        .with(crate::raw_bindings::path::init)
        .with(crate::raw_bindings::http_server::init)
        .with(crate::global_bindings::streams::init)
        .with(crate::global_bindings::compression::init)
        .with(crate::raw_bindings::tcp::init)
        .with(crate::raw_bindings::udp::init)
        .with(crate::raw_bindings::socket::init)
        .with(crate::raw_bindings::wasm::init)
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

    crate::global_bindings::timers::init_registry();

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
