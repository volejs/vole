use rquickjs::{Array, ArrayBuffer, Class, Ctx, IntoJs, Promise, Result, Value};

use crate::classes::CryptoKey;
use crate::key_material::{KeyInfo, KeyMaterial, has_usage};
use crate::provider::{CryptoProvider, HashAlgorithm, default_provider};
use crate::utils::js_value_to_json;
use vole_core::error::{dom_exception_name, reject_dom_exception};

pub(crate) fn crypto_subtle_derive_bits<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    base_key: Class<'js, CryptoKey>,
    length: u32,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let base_key_info = base_key.try_borrow()?.key_info.clone();
    let (promise, resolve, reject) = ctx.promise()?;

    if !has_usage(&base_key_info, "deriveBits") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow deriveBits",
        )?;
        return Ok(promise);
    }

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

        let result = derive_bytes(&algorithm_json, &base_key_info, length);
        match result {
            Ok(bytes) => match ArrayBuffer::new_copy(ctx_clone, bytes) {
                Ok(buffer) => {
                    let _ = resolve.call::<_, ()>((buffer,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("deriveBits output failed",));
                }
            },
            Err(err) => {
                let _ = reject.call::<_, ()>((err,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn crypto_subtle_derive_key<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    base_key: Class<'js, CryptoKey>,
    derived_key_algorithm: Value<'js>,
    extractable: bool,
    usages: Array<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let derived_key_algorithm_json = js_value_to_json(&ctx, &derived_key_algorithm)?;
    let usages_vec = usages.iter::<String>().collect::<Result<Vec<_>>>()?;
    let base_key_info = base_key.try_borrow()?.key_info.clone();
    let (promise, resolve, reject) = ctx.promise()?;

    if !has_usage(&base_key_info, "deriveKey") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow deriveKey",
        )?;
        return Ok(promise);
    }

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

        let result: std::result::Result<Value<'js>, rquickjs::Error> = (|| {
            let length = derived_key_length(&derived_key_algorithm_json)
                .map_err(|_| rquickjs::Error::Exception)?;
            let bytes = derive_bytes(&algorithm_json, &base_key_info, length)
                .map_err(|_| rquickjs::Error::Exception)?;
            let key = derived_crypto_key(
                &ctx_clone,
                bytes,
                &derived_key_algorithm_json,
                extractable,
                usages_vec,
            )?;
            key.into_js(&ctx_clone)
        })();

        match result {
            Ok(value) => {
                let _ = resolve.call::<_, ()>((value,));
            }
            Err(_) => {
                let _ = reject.call::<_, ()>(("deriveKey failed",));
            }
        }
    });

    Ok(promise)
}

fn derive_bytes(
    algorithm: &serde_json::Value,
    base_key_info: &KeyInfo,
    length_bits: u32,
) -> std::result::Result<Vec<u8>, &'static str> {
    if length_bits == 0 || length_bits % 8 != 0 {
        return Err("derived length must be a positive multiple of 8 bits");
    }
    let length = (length_bits / 8) as usize;
    let KeyMaterial::Derive(base_key) = &base_key_info.material else {
        return Err("baseKey must be a derivation key");
    };

    let hash = hash_algorithm(algorithm)?;
    match algorithm["name"].as_str().unwrap_or("") {
        "HKDF" => {
            let salt = bytes_field(algorithm, "salt")?;
            let info = bytes_field(algorithm, "info")?;
            default_provider()
                .hkdf_derive(hash, base_key, &salt, &info, length)
                .map_err(|_| "HKDF derivation failed")
        }
        "PBKDF2" => {
            let salt = bytes_field(algorithm, "salt")?;
            let iterations = algorithm["iterations"]
                .as_u64()
                .ok_or("PBKDF2 iterations is required")?;
            let iterations =
                u32::try_from(iterations).map_err(|_| "PBKDF2 iterations too large")?;
            default_provider()
                .pbkdf2_derive(hash, base_key, &salt, iterations, length)
                .map_err(|_| "PBKDF2 derivation failed")
        }
        _ => Err("Unsupported derivation algorithm"),
    }
}

fn derived_crypto_key<'js>(
    ctx: &Ctx<'js>,
    bytes: Vec<u8>,
    algorithm: &serde_json::Value,
    extractable: bool,
    usages: Vec<String>,
) -> Result<Class<'js, CryptoKey>> {
    let name = algorithm["name"].as_str().unwrap_or("").to_string();
    let (material, stored_algorithm) = match name.as_str() {
        "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
            if !matches!(bytes.len(), 16 | 24 | 32) {
                return Err(rquickjs::Error::Exception);
            }
            (KeyMaterial::Aes(bytes), name.clone())
        }
        "HMAC" => {
            let hash = hash_algorithm(algorithm).map_err(|_| rquickjs::Error::Exception)?;
            let stored = match hash {
                HashAlgorithm::Sha1 => "HMAC-SHA-1",
                HashAlgorithm::Sha256 => "HMAC-SHA-256",
                HashAlgorithm::Sha384 => "HMAC-SHA-384",
                HashAlgorithm::Sha512 => "HMAC-SHA-512",
            };
            (KeyMaterial::Hmac(bytes), stored.to_string())
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    let key_info = KeyInfo {
        material,
        algorithm: stored_algorithm,
        usages: usages.clone(),
        extractable,
    };
    Class::instance(
        ctx.clone(),
        CryptoKey {
            key_info,
            key_type: "secret".to_string(),
            extractable,
            algorithm: algorithm.clone(),
            usages,
        },
    )
}

fn derived_key_length(algorithm: &serde_json::Value) -> std::result::Result<u32, &'static str> {
    match algorithm["name"].as_str().unwrap_or("") {
        "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => algorithm["length"]
            .as_u64()
            .map(|length| length as u32)
            .ok_or("AES derived key length is required"),
        "HMAC" => {
            if let Some(length) = algorithm["length"].as_u64() {
                return Ok(length as u32);
            }
            let hash = hash_algorithm(algorithm)?;
            Ok(match hash {
                HashAlgorithm::Sha1 | HashAlgorithm::Sha256 => 256,
                HashAlgorithm::Sha384 | HashAlgorithm::Sha512 => 512,
            })
        }
        _ => Err("Unsupported derived key algorithm"),
    }
}

fn hash_algorithm(
    algorithm: &serde_json::Value,
) -> std::result::Result<HashAlgorithm, &'static str> {
    let name = algorithm["hash"]
        .as_str()
        .or_else(|| algorithm["hash"]["name"].as_str())
        .ok_or("hash is required")?;
    HashAlgorithm::from_web_name(name).ok_or("Unsupported hash algorithm")
}

fn bytes_field(
    algorithm: &serde_json::Value,
    field: &str,
) -> std::result::Result<Vec<u8>, &'static str> {
    algorithm[field]
        .as_array()
        .ok_or("Missing required BufferSource field")?
        .iter()
        .map(|v| {
            v.as_u64()
                .map(|n| n as u8)
                .ok_or("Invalid BufferSource field")
        })
        .collect()
}
