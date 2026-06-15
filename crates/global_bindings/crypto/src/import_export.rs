use rquickjs::{Array, ArrayBuffer, Class, Ctx, Promise, Result, Value};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};

use crate::classes::CryptoKey;
use crate::key_material::{KeyInfo, KeyMaterial};
use crate::utils::{js_value_to_json, json_value_to_js_value, value_to_bytes};
use vole_core::error::{dom_exception_name, reject_dom_exception, throw_dom_exception};

// =============================================================================
// subtle.importKey / subtle.exportKey
// =============================================================================

pub(crate) fn crypto_subtle_import_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key_data: Value<'js>,
    algorithm: Value<'js>,
    extractable: bool,
    usages: Array<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let name = algorithm_json["name"].as_str().unwrap_or("").to_string();
    let stored_algorithm = if name == "HMAC" {
        hmac_algorithm_name(&algorithm_json).to_string()
    } else {
        name.clone()
    };

    let usages_vec: Vec<String> = usages.iter::<String>().collect::<Result<Vec<_>>>()?;

    let (material, key_type) = match format.as_str() {
        "raw" => {
            let bytes = value_to_bytes(&ctx, &key_data)?;
            let material = match name.as_str() {
                "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
                    validate_aes_key_length(&ctx, bytes.len())?;
                    KeyMaterial::Aes(bytes)
                }
                "HMAC" => KeyMaterial::Hmac(bytes),
                "HKDF" | "PBKDF2" => KeyMaterial::Derive(bytes),
                _ => return Err(rquickjs::Error::Exception),
            };
            (material, "secret")
        }
        "jwk" => {
            let jwk = js_value_to_json(&ctx, &key_data)?;
            let material = import_jwk(&jwk, &name, &stored_algorithm)?;
            (material, "secret")
        }
        "pkcs8" => {
            let bytes = value_to_bytes(&ctx, &key_data)?;
            let _private_key = rsa::RsaPrivateKey::from_pkcs8_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPrivate(bytes), "private")
        }
        "spki" => {
            let bytes = value_to_bytes(&ctx, &key_data)?;
            let _public_key = rsa::RsaPublicKey::from_public_key_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPublic(bytes), "public")
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    let key_info = KeyInfo {
        material,
        algorithm: stored_algorithm,
        usages: usages_vec.clone(),
        extractable,
    };
    let key = CryptoKey {
        key_info,
        key_type: key_type.to_string(),
        extractable,
        algorithm: algorithm_json,
        usages: usages_vec,
    };

    let (promise, resolve, _reject) = ctx.promise()?;
    let instance = Class::instance(ctx.clone(), key)?;
    resolve.call::<_, ()>((instance,))?;
    Ok(promise)
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

pub(crate) fn crypto_subtle_export_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key: Class<'js, CryptoKey>,
) -> Result<Promise<'js>> {
    let key_info = key.try_borrow()?.key_info.clone();

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

    let ctx_clone = ctx.clone();

    ctx.spawn(async move {
        let result: std::result::Result<Value<'js>, rquickjs::Error> = (|| match format.as_str() {
            "raw" => {
                let bytes = match &key_info.material {
                    KeyMaterial::Aes(b) => b.clone(),
                    KeyMaterial::Hmac(b) => b.clone(),
                    _ => return Err(rquickjs::Error::Exception),
                };
                Ok(ArrayBuffer::new_copy(ctx_clone.clone(), bytes)?.into_value())
            }
            "jwk" => {
                let jwk = export_jwk(&key_info)?;
                json_value_to_js_value(&ctx_clone, &jwk)
            }
            "pkcs8" => {
                let bytes = match &key_info.material {
                    KeyMaterial::RsaPrivate(b) => b.clone(),
                    _ => return Err(rquickjs::Error::Exception),
                };
                Ok(ArrayBuffer::new_copy(ctx_clone.clone(), bytes)?.into_value())
            }
            "spki" => {
                let bytes = match &key_info.material {
                    KeyMaterial::RsaPublic(b) => b.clone(),
                    _ => return Err(rquickjs::Error::Exception),
                };
                Ok(ArrayBuffer::new_copy(ctx_clone.clone(), bytes)?.into_value())
            }
            _ => Err(rquickjs::Error::Exception),
        })();

        match result {
            Ok(value) => {
                let _ = resolve.call::<_, ()>((value,));
            }
            Err(_) => {
                let _ = reject.call::<_, ()>(("exportKey failed",));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn import_jwk(
    jwk: &serde_json::Value,
    expected_name: &str,
    expected_algorithm: &str,
) -> Result<KeyMaterial> {
    let kty = jwk["kty"].as_str().ok_or(rquickjs::Error::Exception)?;
    if kty != "oct" {
        return Err(rquickjs::Error::Exception);
    }

    let k = jwk["k"].as_str().ok_or(rquickjs::Error::Exception)?;
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(k)
        .map_err(|_| rquickjs::Error::Exception)?;

    if let Some(alg) = jwk["alg"].as_str() {
        let expected_alg = match expected_name {
            "AES-CBC" => match bytes.len() {
                16 => "A128CBC",
                24 => "A192CBC",
                32 => "A256CBC",
                _ => return Err(rquickjs::Error::Exception),
            },
            "AES-CTR" => match bytes.len() {
                16 => "A128CTR",
                24 => "A192CTR",
                32 => "A256CTR",
                _ => return Err(rquickjs::Error::Exception),
            },
            "AES-GCM" => match bytes.len() {
                16 => "A128GCM",
                24 => "A192GCM",
                32 => "A256GCM",
                _ => return Err(rquickjs::Error::Exception),
            },
            "AES-KW" => match bytes.len() {
                16 => "A128KW",
                24 => "A192KW",
                32 => "A256KW",
                _ => return Err(rquickjs::Error::Exception),
            },
            "HMAC" => match expected_algorithm {
                "HMAC-SHA-1" => "HS1",
                "HMAC-SHA-256" => "HS256",
                "HMAC-SHA-384" => "HS384",
                "HMAC-SHA-512" => "HS512",
                _ => return Err(rquickjs::Error::Exception),
            },
            _ => return Err(rquickjs::Error::Exception),
        };
        if alg != expected_alg {
            return Err(rquickjs::Error::Exception);
        }
    }

    match expected_name {
        "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => Ok(KeyMaterial::Aes(bytes)),
        "HMAC" => Ok(KeyMaterial::Hmac(bytes)),
        _ => Err(rquickjs::Error::Exception),
    }
}

pub(crate) fn export_jwk(key_info: &KeyInfo) -> Result<serde_json::Value> {
    use base64::Engine;
    let bytes = match &key_info.material {
        KeyMaterial::Aes(b) => b,
        KeyMaterial::Hmac(b) => b,
        KeyMaterial::RsaPrivate(_) | KeyMaterial::RsaPublic(_) | KeyMaterial::Derive(_) => {
            return Err(rquickjs::Error::Exception);
        }
    };
    let k = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let alg = match key_info.algorithm.as_str() {
        "AES-CBC" => match bytes.len() {
            16 => "A128CBC",
            24 => "A192CBC",
            32 => "A256CBC",
            _ => return Err(rquickjs::Error::Exception),
        },
        "AES-CTR" => match bytes.len() {
            16 => "A128CTR",
            24 => "A192CTR",
            32 => "A256CTR",
            _ => return Err(rquickjs::Error::Exception),
        },
        "AES-GCM" => match bytes.len() {
            16 => "A128GCM",
            24 => "A192GCM",
            32 => "A256GCM",
            _ => return Err(rquickjs::Error::Exception),
        },
        "AES-KW" => match bytes.len() {
            16 => "A128KW",
            24 => "A192KW",
            32 => "A256KW",
            _ => return Err(rquickjs::Error::Exception),
        },
        "HMAC-SHA-1" => "HS1",
        "HMAC-SHA-256" => "HS256",
        "HMAC-SHA-384" => "HS384",
        "HMAC-SHA-512" => "HS512",
        _ => return Err(rquickjs::Error::Exception),
    };

    let mut jwk = serde_json::Map::new();
    jwk.insert(
        "kty".to_string(),
        serde_json::Value::String("oct".to_string()),
    );
    jwk.insert("k".to_string(), serde_json::Value::String(k));
    jwk.insert(
        "alg".to_string(),
        serde_json::Value::String(alg.to_string()),
    );
    jwk.insert(
        "key_ops".to_string(),
        serde_json::Value::Array(
            key_info
                .usages
                .iter()
                .map(|u| serde_json::Value::String(u.clone()))
                .collect(),
        ),
    );
    jwk.insert(
        "ext".to_string(),
        serde_json::Value::Bool(key_info.extractable),
    );
    Ok(serde_json::Value::Object(jwk))
}
