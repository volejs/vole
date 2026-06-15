use rquickjs::{ArrayBuffer, Class, Ctx, Promise, Result, Value};

use crate::classes::CryptoKey;
use crate::key_material::{KeyInfo, KeyMaterial, has_usage};
use crate::provider::{AesMode, CryptoError, CryptoProvider, default_provider};
use crate::utils::{js_value_to_json, value_to_bytes};
use vole_core::error::{dom_exception_name, reject_dom_exception};

// =============================================================================
// subtle.encrypt / subtle.decrypt
// =============================================================================

pub(crate) fn crypto_subtle_encrypt<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    key: Class<'js, CryptoKey>,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let plaintext = value_to_bytes(&ctx, &data)?;
    let key_info = key.try_borrow()?.key_info.clone();

    let (promise, resolve, reject) = ctx.promise()?;
    if !has_usage(&key_info, "encrypt") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow encrypt",
        )?;
        return Ok(promise);
    }
    let ctx_clone = ctx.clone();

    ctx.spawn(async move {
        let result = do_encrypt(&key_info, &algorithm_json, &plaintext);
        match result {
            Ok(ciphertext) => match ArrayBuffer::new_copy(ctx_clone, ciphertext) {
                Ok(ta) => {
                    let _ = resolve.call::<_, ()>((ta,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("encrypt output failed",));
                }
            },
            Err(e) => {
                let _ = reject.call::<_, ()>((e,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn crypto_subtle_decrypt<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    key: Class<'js, CryptoKey>,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let ciphertext = value_to_bytes(&ctx, &data)?;
    let key_info = key.try_borrow()?.key_info.clone();

    let (promise, resolve, reject) = ctx.promise()?;
    if !has_usage(&key_info, "decrypt") {
        reject_dom_exception(
            &ctx,
            &reject,
            dom_exception_name::INVALID_ACCESS,
            "CryptoKey does not allow decrypt",
        )?;
        return Ok(promise);
    }
    let ctx_clone = ctx.clone();

    ctx.spawn(async move {
        let result = do_decrypt(&key_info, &algorithm_json, &ciphertext);
        match result {
            Ok(plaintext) => match ArrayBuffer::new_copy(ctx_clone, plaintext) {
                Ok(ta) => {
                    let _ = resolve.call::<_, ()>((ta,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("decrypt output failed",));
                }
            },
            Err(e) => {
                let _ = reject.call::<_, ()>((e,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn do_encrypt(
    key_info: &KeyInfo,
    algorithm: &serde_json::Value,
    plaintext: &[u8],
) -> std::result::Result<Vec<u8>, &'static str> {
    match &key_info.material {
        KeyMaterial::Aes(key_bytes) => {
            let name = algorithm["name"].as_str().unwrap_or("");
            match name {
                "AES-CBC" => {
                    let iv = required_bytes_field(algorithm, "iv")?;
                    default_provider()
                        .aes_encrypt(AesMode::Cbc, key_bytes, &iv, plaintext, None)
                        .map_err(provider_error_message)
                }
                "AES-CTR" => {
                    let counter = required_bytes_field(algorithm, "counter")?;
                    let counter_length = aes_ctr_counter_length(algorithm)?;
                    default_provider()
                        .aes_encrypt(
                            AesMode::Ctr { counter_length },
                            key_bytes,
                            &counter,
                            plaintext,
                            None,
                        )
                        .map_err(provider_error_message)
                }
                "AES-GCM" => {
                    ensure_gcm_tag_length(algorithm)?;
                    let additional_data = optional_bytes_field(algorithm, "additionalData")?;
                    let iv = required_bytes_field(algorithm, "iv")?;
                    default_provider()
                        .aes_encrypt(
                            AesMode::Gcm,
                            key_bytes,
                            &iv,
                            plaintext,
                            Some(additional_data.as_slice()),
                        )
                        .map_err(provider_error_message)
                }
                _ => Err("Unsupported AES mode"),
            }
        }
        _ => Err("Invalid key type for encryption"),
    }
}

pub(crate) fn do_decrypt(
    key_info: &KeyInfo,
    algorithm: &serde_json::Value,
    ciphertext: &[u8],
) -> std::result::Result<Vec<u8>, &'static str> {
    match &key_info.material {
        KeyMaterial::Aes(key_bytes) => {
            let name = algorithm["name"].as_str().unwrap_or("");
            match name {
                "AES-CBC" => {
                    let iv = required_bytes_field(algorithm, "iv")?;
                    default_provider()
                        .aes_decrypt(AesMode::Cbc, key_bytes, &iv, ciphertext, None)
                        .map_err(provider_error_message)
                }
                "AES-CTR" => {
                    let counter = required_bytes_field(algorithm, "counter")?;
                    let counter_length = aes_ctr_counter_length(algorithm)?;
                    default_provider()
                        .aes_decrypt(
                            AesMode::Ctr { counter_length },
                            key_bytes,
                            &counter,
                            ciphertext,
                            None,
                        )
                        .map_err(provider_error_message)
                }
                "AES-GCM" => {
                    ensure_gcm_tag_length(algorithm)?;
                    let additional_data = optional_bytes_field(algorithm, "additionalData")?;
                    let iv = required_bytes_field(algorithm, "iv")?;
                    default_provider()
                        .aes_decrypt(
                            AesMode::Gcm,
                            key_bytes,
                            &iv,
                            ciphertext,
                            Some(additional_data.as_slice()),
                        )
                        .map_err(provider_error_message)
                }
                _ => Err("Unsupported AES mode"),
            }
        }
        _ => Err("Invalid key type for decryption"),
    }
}

fn required_bytes_field(
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

fn provider_error_message(error: CryptoError) -> &'static str {
    match error {
        CryptoError::InvalidLength => "Invalid key or IV length",
        CryptoError::OperationFailed => "Crypto operation failed",
    }
}

fn aes_ctr_counter_length(algorithm: &serde_json::Value) -> std::result::Result<u32, &'static str> {
    let length = algorithm["length"]
        .as_u64()
        .ok_or("AES-CTR length is required")?;
    if !(1..=128).contains(&length) {
        return Err("AES-CTR length must be between 1 and 128 bits");
    }
    Ok(length as u32)
}

fn ensure_gcm_tag_length(algorithm: &serde_json::Value) -> std::result::Result<(), &'static str> {
    let tag_length = algorithm["tagLength"].as_u64().unwrap_or(128);
    if tag_length == 128 {
        return Ok(());
    }
    Err("AES-GCM tagLength values other than 128 are not supported")
}

fn optional_bytes_field(
    algorithm: &serde_json::Value,
    name: &str,
) -> std::result::Result<Vec<u8>, &'static str> {
    let Some(value) = algorithm.get(name) else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let Some(array) = value.as_array() else {
        return Err("Algorithm byte field must be a BufferSource");
    };
    array
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or("Algorithm byte field contains invalid byte")
        })
        .collect()
}
