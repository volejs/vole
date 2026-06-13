use rquickjs::{Array, ArrayBuffer, Class, Ctx, Promise, Result, Value};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};

use crate::classes::CryptoKey;
use crate::key_material::{store_key, KeyInfo, KeyMaterial};
use crate::utils::{js_value_to_json, json_value_to_js_value, value_to_bytes};

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
    let name = algorithm_json["name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let usages_vec: Vec<String> = usages.iter::<String>().collect::<Result<Vec<_>>>()?;

    let (material, key_type) = match format.as_str() {
        "raw" => {
            let bytes = value_to_bytes(&key_data)?;
            let material = match name.as_str() {
                "AES-CBC" | "AES-GCM" => KeyMaterial::Aes(bytes),
                "HMAC" => KeyMaterial::Hmac(bytes),
                _ => return Err(rquickjs::Error::Exception),
            };
            (material, "secret")
        }
        "jwk" => {
            let jwk = js_value_to_json(&ctx, &key_data)?;
            let material = import_jwk(&jwk, &name)?;
            (material, "secret")
        }
        "pkcs8" => {
            let bytes = value_to_bytes(&key_data)?;
            let _private_key = rsa::RsaPrivateKey::from_pkcs8_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPrivate(bytes), "private")
        }
        "spki" => {
            let bytes = value_to_bytes(&key_data)?;
            let _public_key = rsa::RsaPublicKey::from_public_key_der(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            (KeyMaterial::RsaPublic(bytes), "public")
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    let id = store_key(KeyInfo {
        material,
        algorithm: name.clone(),
        usages: usages_vec.clone(),
        extractable,
    });

    let key = CryptoKey {
        id,
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

pub(crate) fn crypto_subtle_export_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key: Class<'js, CryptoKey>,
) -> Result<Promise<'js>> {
    let key_info = crate::key_material::get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
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

pub(crate) fn import_jwk(jwk: &serde_json::Value, expected_name: &str) -> Result<KeyMaterial> {
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
            "AES-GCM" => match bytes.len() {
                16 => "A128GCM",
                24 => "A192GCM",
                32 => "A256GCM",
                _ => return Err(rquickjs::Error::Exception),
            },
            "HMAC" => match bytes.len() {
                32 => "HS256",
                48 => "HS384",
                64 => "HS512",
                _ => return Err(rquickjs::Error::Exception),
            },
            _ => return Err(rquickjs::Error::Exception),
        };
        if alg != expected_alg {
            return Err(rquickjs::Error::Exception);
        }
    }

    match expected_name {
        "AES-CBC" | "AES-GCM" => Ok(KeyMaterial::Aes(bytes)),
        "HMAC" => Ok(KeyMaterial::Hmac(bytes)),
        _ => Err(rquickjs::Error::Exception),
    }
}

pub(crate) fn export_jwk(key_info: &KeyInfo) -> Result<serde_json::Value> {
    use base64::Engine;
    let bytes = match &key_info.material {
        KeyMaterial::Aes(b) => b,
        KeyMaterial::Hmac(b) => b,
        KeyMaterial::RsaPrivate(_) | KeyMaterial::RsaPublic(_) => {
            return Err(rquickjs::Error::Exception)
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
        "AES-GCM" => match bytes.len() {
            16 => "A128GCM",
            24 => "A192GCM",
            32 => "A256GCM",
            _ => return Err(rquickjs::Error::Exception),
        },
        alg if alg.starts_with("HMAC-") => match bytes.len() {
            32 => "HS256",
            48 => "HS384",
            64 => "HS512",
            _ => return Err(rquickjs::Error::Exception),
        },
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
