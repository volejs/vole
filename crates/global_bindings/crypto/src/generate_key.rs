use rquickjs::{Class, Ctx, IntoJs, Object, Promise, Result, Value};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

use crate::classes::CryptoKey;
use crate::key_material::{filter_public_usages, KeyInfo, KeyMaterial};
use crate::provider::{default_provider, CryptoProvider};
use crate::utils::js_value_to_json;
use vole_core::error::{dom_exception_name, reject_dom_exception};

// =============================================================================
// subtle.generateKey
// =============================================================================

pub(crate) fn crypto_subtle_generate_key<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    extractable: bool,
    usages: rquickjs::Array<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let name = algorithm_json["name"].as_str().unwrap_or("").to_string();

    let usages_vec: Vec<String> = usages.iter::<String>().collect::<Result<Vec<_>>>()?;

    let (promise, resolve, reject) = ctx.promise()?;
    if matches!(name.as_str(), "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW") {
        let length = algorithm_json["length"].as_u64().unwrap_or(256);
        if !matches!(length, 128 | 192 | 256) {
            reject_dom_exception(
                &ctx,
                &reject,
                dom_exception_name::DATA,
                "AES key length must be 128, 192, or 256 bits",
            )?;
            return Ok(promise);
        }
    }
    if name == "HMAC" {
        let hash = hmac_hash_name(&algorithm_json);
        if !matches!(hash, "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512") {
            reject_dom_exception(
                &ctx,
                &reject,
                dom_exception_name::NOT_SUPPORTED,
                "Unsupported HMAC hash algorithm",
            )?;
            return Ok(promise);
        }
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

        let result: std::result::Result<Value<'js>, rquickjs::Error> = (|| match name.as_str() {
            "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
                let length = algorithm_json["length"].as_u64().unwrap_or(256) as usize;
                let key_len = length / 8;
                let mut key_bytes = vec![0u8; key_len];
                default_provider()
                    .fill_random(&mut key_bytes)
                    .map_err(|_| rquickjs::Error::Exception)?;
                let key_info = KeyInfo {
                    material: KeyMaterial::Aes(key_bytes),
                    algorithm: name.clone(),
                    usages: usages_vec.clone(),
                    extractable,
                };
                let key = CryptoKey {
                    key_info,
                    key_type: "secret".to_string(),
                    extractable,
                    algorithm: algorithm_json.clone(),
                    usages: usages_vec.clone(),
                };
                Class::instance(ctx_clone.clone(), key).into_js(&ctx_clone)
            }
            "HMAC" => {
                let hash = hmac_hash_name(&algorithm_json);
                let key_len = match hash {
                    "SHA-1" | "SHA-256" => 32,
                    "SHA-384" | "SHA-512" => 64,
                    _ => 32,
                };
                let mut key_bytes = vec![0u8; key_len];
                default_provider()
                    .fill_random(&mut key_bytes)
                    .map_err(|_| rquickjs::Error::Exception)?;
                let key_info = KeyInfo {
                    material: KeyMaterial::Hmac(key_bytes),
                    algorithm: format!("HMAC-{}", hash),
                    usages: usages_vec.clone(),
                    extractable,
                };
                let key = CryptoKey {
                    key_info,
                    key_type: "secret".to_string(),
                    extractable,
                    algorithm: algorithm_json.clone(),
                    usages: usages_vec.clone(),
                };
                Class::instance(ctx_clone.clone(), key).into_js(&ctx_clone)
            }
            "RSASSA-PKCS1-v1.5" | "RSA-PSS" => {
                let bits = algorithm_json["modulusLength"].as_u64().unwrap_or(2048) as usize;
                let private_key = rsa::RsaPrivateKey::new(&mut rand::thread_rng(), bits)
                    .map_err(|_| rquickjs::Error::Exception)?;
                let public_key = rsa::RsaPublicKey::from(&private_key);
                let private_der = private_key
                    .to_pkcs8_der()
                    .map_err(|_| rquickjs::Error::Exception)?;
                let public_der = public_key
                    .to_public_key_der()
                    .map_err(|_| rquickjs::Error::Exception)?;

                let private_key_info = KeyInfo {
                    material: KeyMaterial::RsaPrivate(private_der.as_bytes().to_vec()),
                    algorithm: name.clone(),
                    usages: usages_vec.clone(),
                    extractable,
                };
                let public_key_info = KeyInfo {
                    material: KeyMaterial::RsaPublic(public_der.as_bytes().to_vec()),
                    algorithm: name.clone(),
                    usages: filter_public_usages(&usages_vec),
                    extractable,
                };
                let private_key = CryptoKey {
                    key_info: private_key_info,
                    key_type: "private".to_string(),
                    extractable,
                    algorithm: algorithm_json.clone(),
                    usages: usages_vec.clone(),
                };
                let public_key = CryptoKey {
                    key_info: public_key_info,
                    key_type: "public".to_string(),
                    extractable,
                    algorithm: algorithm_json.clone(),
                    usages: filter_public_usages(&usages_vec),
                };

                let pair = Object::new(ctx_clone.clone())?;
                pair.set(
                    "privateKey",
                    Class::instance(ctx_clone.clone(), private_key)?,
                )?;
                pair.set("publicKey", Class::instance(ctx_clone.clone(), public_key)?)?;
                Ok(pair.into_value())
            }
            _ => Err(rquickjs::Error::Exception),
        })();

        match result {
            Ok(value) => {
                let _ = resolve.call::<_, ()>((value,));
            }
            Err(_) => {
                let _ = reject.call::<_, ()>(("generateKey failed",));
            }
        }
    });

    Ok(promise)
}

fn hmac_hash_name(algorithm: &serde_json::Value) -> &'static str {
    match algorithm["hash"]
        .as_str()
        .or_else(|| algorithm["hash"]["name"].as_str())
        .unwrap_or("SHA-256")
    {
        "SHA-1" => "SHA-1",
        "SHA-384" => "SHA-384",
        "SHA-512" => "SHA-512",
        _ => "SHA-256",
    }
}
