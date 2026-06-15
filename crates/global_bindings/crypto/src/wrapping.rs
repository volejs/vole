use rquickjs::{
    Array, ArrayBuffer, Class, Coerced, Ctx, Exception, FromJs, IntoJs, Promise, Result, Value,
    function::Rest,
};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};

use crate::cipher::{do_decrypt, do_encrypt};
use crate::classes::CryptoKey;
use crate::import_export::{export_jwk, import_jwk};
use crate::key_material::{KeyInfo, KeyMaterial, has_usage};
use crate::provider::{CryptoProvider, default_provider};
use crate::utils::{js_value_to_json, value_to_bytes};
use vole_core::error::{dom_exception_name, reject_dom_exception, throw_dom_exception};

pub(crate) fn crypto_subtle_wrap_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key: Class<'js, CryptoKey>,
    wrapping_key: Class<'js, CryptoKey>,
    wrap_algorithm: Value<'js>,
) -> Result<Promise<'js>> {
    let key_info = key.try_borrow()?.key_info.clone();
    let wrapping_key_info = wrapping_key.try_borrow()?.key_info.clone();
    let wrap_algorithm_json = js_value_to_json(&ctx, &wrap_algorithm)?;

    let (promise, resolve, reject) = ctx.promise()?;
    if !key_info.extractable {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey is not extractable",
        )?;
        return Ok(promise);
    }
    if !has_usage(&wrapping_key_info, "wrapKey") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow wrapKey",
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

        let result = wrap_key_bytes(&format, &key_info, &wrapping_key_info, &wrap_algorithm_json);
        match result {
            Ok(bytes) => match ArrayBuffer::new_copy(ctx_clone, bytes) {
                Ok(buffer) => {
                    let _ = resolve.call::<_, ()>((buffer,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("wrapKey output failed",));
                }
            },
            Err(err) => {
                let _ = reject.call::<_, ()>((err,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn crypto_subtle_unwrap_key<'js>(
    ctx: Ctx<'js>,
    args: Rest<Value<'js>>,
) -> Result<Promise<'js>> {
    let mut args = args.0.into_iter();
    let format = required_arg(&ctx, args.next(), "format")?;
    let format = Coerced::<String>::from_js(&ctx, format)?.0;
    let wrapped_key = required_arg(&ctx, args.next(), "wrappedKey")?;
    let wrapped_key = value_to_bytes(&ctx, &wrapped_key)?;
    let unwrapping_key =
        Class::<CryptoKey>::from_js(&ctx, required_arg(&ctx, args.next(), "unwrappingKey")?)?;
    let unwrap_algorithm = required_arg(&ctx, args.next(), "unwrapAlgorithm")?;
    let unwrap_algorithm_json = js_value_to_json(&ctx, &unwrap_algorithm)?;
    let unwrapped_key_algorithm = required_arg(&ctx, args.next(), "unwrappedKeyAlgorithm")?;
    let unwrapped_key_algorithm_json = js_value_to_json(&ctx, &unwrapped_key_algorithm)?;
    let extractable = bool::from_js(&ctx, required_arg(&ctx, args.next(), "extractable")?)?;
    let usages = Array::from_js(&ctx, required_arg(&ctx, args.next(), "keyUsages")?)?;
    let usages_vec = usages.iter::<String>().collect::<Result<Vec<_>>>()?;
    let unwrapping_key_info = unwrapping_key.try_borrow()?.key_info.clone();

    let (promise, resolve, reject) = ctx.promise()?;
    if !has_usage(&unwrapping_key_info, "unwrapKey") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow unwrapKey",
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
            let bytes =
                unwrap_key_bytes(&wrapped_key, &unwrapping_key_info, &unwrap_algorithm_json)
                    .map_err(|_| rquickjs::Error::Exception)?;
            let key = import_unwrapped_key(
                &ctx_clone,
                &format,
                bytes,
                &unwrapped_key_algorithm_json,
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
                let _ = reject.call::<_, ()>(("unwrapKey failed",));
            }
        }
    });

    Ok(promise)
}

fn required_arg<'js>(ctx: &Ctx<'js>, value: Option<Value<'js>>, name: &str) -> Result<Value<'js>> {
    value.ok_or_else(|| Exception::throw_type(ctx, &format!("{name} is required")))
}

fn wrap_key_bytes(
    format: &str,
    key_info: &KeyInfo,
    wrapping_key_info: &KeyInfo,
    wrap_algorithm: &serde_json::Value,
) -> std::result::Result<Vec<u8>, &'static str> {
    let exported = export_key_bytes(format, key_info)?;
    if wrap_algorithm["name"].as_str().unwrap_or("") == "AES-KW" {
        let KeyMaterial::Aes(kek) = &wrapping_key_info.material else {
            return Err("Invalid key type for AES-KW");
        };
        return default_provider()
            .aes_kw_wrap(kek, &exported)
            .map_err(|_| "AES-KW wrap failed");
    }
    do_encrypt(wrapping_key_info, wrap_algorithm, &exported)
}

fn unwrap_key_bytes(
    wrapped_key: &[u8],
    unwrapping_key_info: &KeyInfo,
    unwrap_algorithm: &serde_json::Value,
) -> std::result::Result<Vec<u8>, &'static str> {
    if unwrap_algorithm["name"].as_str().unwrap_or("") == "AES-KW" {
        let KeyMaterial::Aes(kek) = &unwrapping_key_info.material else {
            return Err("Invalid key type for AES-KW");
        };
        return default_provider()
            .aes_kw_unwrap(kek, wrapped_key)
            .map_err(|_| "AES-KW unwrap failed");
    }
    do_decrypt(unwrapping_key_info, unwrap_algorithm, wrapped_key)
}

fn export_key_bytes(
    format: &str,
    key_info: &KeyInfo,
) -> std::result::Result<Vec<u8>, &'static str> {
    match format {
        "raw" => match &key_info.material {
            KeyMaterial::Aes(bytes) | KeyMaterial::Hmac(bytes) => Ok(bytes.clone()),
            _ => Err("raw export is not supported for this key type"),
        },
        "jwk" => {
            let jwk = export_jwk(key_info).map_err(|_| "jwk export failed")?;
            serde_json::to_vec(&jwk).map_err(|_| "jwk serialization failed")
        }
        "pkcs8" => match &key_info.material {
            KeyMaterial::RsaPrivate(bytes) => Ok(bytes.clone()),
            _ => Err("pkcs8 export requires a private key"),
        },
        "spki" => match &key_info.material {
            KeyMaterial::RsaPublic(bytes) => Ok(bytes.clone()),
            _ => Err("spki export requires a public key"),
        },
        _ => Err("Unsupported key format"),
    }
}

fn import_unwrapped_key<'js>(
    ctx: &Ctx<'js>,
    format: &str,
    bytes: Vec<u8>,
    algorithm: &serde_json::Value,
    extractable: bool,
    usages: Vec<String>,
) -> Result<Class<'js, CryptoKey>> {
    let name = algorithm["name"].as_str().unwrap_or("").to_string();
    let stored_algorithm = if name == "HMAC" {
        hmac_algorithm_name(algorithm).to_string()
    } else {
        name.clone()
    };
    let (material, key_type) = match format {
        "raw" => {
            let material = match name.as_str() {
                "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
                    validate_aes_key_length(ctx, bytes.len())?;
                    KeyMaterial::Aes(bytes)
                }
                "HMAC" => KeyMaterial::Hmac(bytes),
                _ => return Err(rquickjs::Error::Exception),
            };
            (material, "secret")
        }
        "jwk" => {
            let jwk = serde_json::from_slice::<serde_json::Value>(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            let material = import_jwk(&jwk, &name, &stored_algorithm)?;
            (material, "secret")
        }
        "pkcs8" => {
            let _private_key = rsa::RsaPrivateKey::from_pkcs8_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPrivate(bytes), "private")
        }
        "spki" => {
            let _public_key = rsa::RsaPublicKey::from_public_key_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPublic(bytes), "public")
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
            key_type: key_type.to_string(),
            extractable,
            algorithm: algorithm.clone(),
            usages,
        },
    )
}

fn hmac_algorithm_name(algorithm: &serde_json::Value) -> &'static str {
    let hash = algorithm["hash"]
        .as_str()
        .or_else(|| algorithm["hash"]["name"].as_str())
        .unwrap_or("SHA-256");
    match hash {
        "SHA-1" => "HMAC-SHA-1",
        "SHA-384" => "HMAC-SHA-384",
        "SHA-512" => "HMAC-SHA-512",
        _ => "HMAC-SHA-256",
    }
}

fn validate_aes_key_length<'js>(ctx: &Ctx<'js>, len: usize) -> Result<()> {
    if matches!(len, 16 | 24 | 32) {
        return Ok(());
    }
    throw_dom_exception(
        ctx,
        dom_exception_name::DATA,
        "AES key length must be 128, 192, or 256 bits",
    )
}
