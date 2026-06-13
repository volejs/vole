use rquickjs::{ArrayBuffer, Ctx, Promise, Result, Value};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha384, Sha512};

use crate::utils::{js_value_to_json, value_to_bytes};

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

    let bytes = value_to_bytes(&data)?;

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

        let result = match alg_name.as_str() {
            "SHA-1" => {
                let mut hasher = Sha1::new();
                hasher.update(&bytes);
                hasher.finalize().to_vec()
            }
            "SHA-256" => {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                hasher.finalize().to_vec()
            }
            "SHA-384" => {
                let mut hasher = Sha384::new();
                hasher.update(&bytes);
                hasher.finalize().to_vec()
            }
            "SHA-512" => {
                let mut hasher = Sha512::new();
                hasher.update(&bytes);
                hasher.finalize().to_vec()
            }
            _ => {
                let _ = reject.call::<_, ()>((ctx_clone.clone().catch(),));
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
