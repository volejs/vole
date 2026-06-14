use rquickjs::{ArrayBuffer, Ctx, Promise, Result, Value};

use crate::provider::{default_provider, CryptoProvider, HashAlgorithm};
use crate::utils::{js_value_to_json, value_to_bytes};
use vole_core::error::{dom_exception_name, reject_dom_exception};

// =============================================================================
// subtle.digest
// =============================================================================

pub(crate) fn crypto_subtle_digest<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let alg = js_value_to_json(&ctx, &algorithm)?;
    let alg_name = match &alg {
        serde_json::Value::String(s) => s.clone(),
        _ => alg["name"].as_str().unwrap_or("").to_string(),
    };

    let bytes = value_to_bytes(&ctx, &data)?;

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    vole_global_timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                vole_global_timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let result = match HashAlgorithm::from_web_name(alg_name.as_str()) {
            Some(algorithm) => default_provider().digest(algorithm, &bytes),
            None => {
                let _ = reject_dom_exception(
                    &ctx_clone,
                    &reject,
                    dom_exception_name::NOT_SUPPORTED,
                    "Unrecognized digest algorithm",
                );
                return;
            }
        };

        match ArrayBuffer::new_copy(ctx_clone.clone(), result) {
            Ok(array) => {
                let _ = resolve.call::<_, ()>((array,));
            }
            Err(_) => {
                let _ = reject.call::<_, ()>((ctx_clone.catch(),));
            }
        }
    });

    Ok(promise)
}
