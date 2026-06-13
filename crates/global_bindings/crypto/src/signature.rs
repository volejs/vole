use rquickjs::{ArrayBuffer, Class, Ctx, Promise, Result, Value};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::classes::CryptoKey;
use crate::key_material::{get_key, KeyInfo, KeyMaterial};
use crate::utils::{js_value_to_json, value_to_bytes};

// =============================================================================
// subtle.sign / subtle.verify
// =============================================================================

pub(crate) fn crypto_subtle_sign<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    key: Class<'js, CryptoKey>,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let message = value_to_bytes(&data)?;
    let key_info = get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

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

        let result = do_sign(&key_info, &algorithm_json, &message);
        match result {
            Ok(sig) => match ArrayBuffer::new_copy(ctx_clone, sig) {
                Ok(ab) => {
                    let _ = resolve.call::<_, ()>((ab,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("sign output failed",));
                }
            },
            Err(e) => {
                let _ = reject.call::<_, ()>((e,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn crypto_subtle_verify<'js>(
    ctx: Ctx<'js>,
    algorithm: Value<'js>,
    key: Class<'js, CryptoKey>,
    signature: Value<'js>,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let algorithm_json = js_value_to_json(&ctx, &algorithm)?;
    let sig = value_to_bytes(&signature)?;
    let message = value_to_bytes(&data)?;
    let key_info = get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;

    vole_global_timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                vole_global_timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let result = do_verify(&key_info, &algorithm_json, &message, &sig);
        match result {
            Ok(valid) => {
                let _ = resolve.call::<_, ()>((valid,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e,));
            }
        }
    });

    Ok(promise)
}

pub(crate) fn hash_name(algorithm: &serde_json::Value) -> &'static str {
    match algorithm["hash"]["name"].as_str() {
        Some("SHA-1") => "SHA-1",
        Some("SHA-256") => "SHA-256",
        Some("SHA-384") => "SHA-384",
        Some("SHA-512") => "SHA-512",
        _ => "SHA-256",
    }
}

pub(crate) fn do_sign(
    key_info: &KeyInfo,
    algorithm: &serde_json::Value,
    message: &[u8],
) -> std::result::Result<Vec<u8>, &'static str> {
    match &key_info.material {
        KeyMaterial::Hmac(key_bytes) => {
            let hash_name = hash_name(algorithm);
            hmac_sign(key_bytes, hash_name, message)
        }
        KeyMaterial::RsaPrivate(der) => {
            let private_key =
                rsa::RsaPrivateKey::from_pkcs8_der(der).map_err(|_| "invalid RSA private key")?;
            let hash_name = hash_name(algorithm);
            let alg_name = key_info.algorithm.as_str();
            rsa_sign(&private_key, alg_name, hash_name, message)
        }
        _ => Err("Invalid key type for signing"),
    }
}

pub(crate) fn do_verify(
    key_info: &KeyInfo,
    algorithm: &serde_json::Value,
    message: &[u8],
    signature: &[u8],
) -> std::result::Result<bool, &'static str> {
    match &key_info.material {
        KeyMaterial::Hmac(key_bytes) => {
            let hash_name = hash_name(algorithm);
            let expected = hmac_sign(key_bytes, hash_name, message)?;
            Ok(expected == signature)
        }
        KeyMaterial::RsaPublic(der) => {
            let public_key = rsa::RsaPublicKey::from_public_key_der(der)
                .map_err(|_| "invalid RSA public key")?;
            let hash_name = hash_name(algorithm);
            let alg_name = key_info.algorithm.as_str();
            rsa_verify(&public_key, alg_name, hash_name, message, signature)
        }
        KeyMaterial::RsaPrivate(der) => {
            let private_key =
                rsa::RsaPrivateKey::from_pkcs8_der(der).map_err(|_| "invalid RSA private key")?;
            let public_key = rsa::RsaPublicKey::from(&private_key);
            let hash_name = hash_name(algorithm);
            let alg_name = key_info.algorithm.as_str();
            rsa_verify(&public_key, alg_name, hash_name, message, signature)
        }
        _ => Err("Invalid key type for verification"),
    }
}

pub(crate) fn hmac_sign(
    key_bytes: &[u8],
    hash_name: &str,
    message: &[u8],
) -> std::result::Result<Vec<u8>, &'static str> {
    use hmac::{Hmac, Mac};
    match hash_name {
        "SHA-1" => {
            type HmacSha = Hmac<Sha1>;
            let mut mac = HmacSha::new_from_slice(key_bytes).map_err(|_| "Invalid key length")?;
            mac.update(message);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "SHA-256" => {
            type HmacSha = Hmac<Sha256>;
            let mut mac = HmacSha::new_from_slice(key_bytes).map_err(|_| "Invalid key length")?;
            mac.update(message);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "SHA-384" => {
            type HmacSha = Hmac<Sha384>;
            let mut mac = HmacSha::new_from_slice(key_bytes).map_err(|_| "Invalid key length")?;
            mac.update(message);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "SHA-512" => {
            type HmacSha = Hmac<Sha512>;
            let mut mac = HmacSha::new_from_slice(key_bytes).map_err(|_| "Invalid key length")?;
            mac.update(message);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        _ => Err("unsupported hash algorithm"),
    }
}

pub(crate) fn rsa_sign(
    private_key: &rsa::RsaPrivateKey,
    algorithm_name: &str,
    hash_name: &str,
    message: &[u8],
) -> std::result::Result<Vec<u8>, &'static str> {
    use rsa::signature::{SignatureEncoding, Signer};
    match algorithm_name {
        "RSASSA-PKCS1-v1.5" => match hash_name {
            "SHA-1" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<sha1::Sha1>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-256" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha256>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-384" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha384>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-512" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha512>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            _ => Err("unsupported hash algorithm"),
        },
        "RSA-PSS" => match hash_name {
            "SHA-1" => {
                let signing_key = rsa::pss::SigningKey::<sha1::Sha1>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-256" => {
                let signing_key = rsa::pss::SigningKey::<Sha256>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-384" => {
                let signing_key = rsa::pss::SigningKey::<Sha384>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            "SHA-512" => {
                let signing_key = rsa::pss::SigningKey::<Sha512>::new(private_key.clone());
                Ok(signing_key
                    .try_sign(message)
                    .map_err(|_| "sign failed")?
                    .to_vec())
            }
            _ => Err("unsupported hash algorithm"),
        },
        _ => Err("invalid RSA signing algorithm"),
    }
}

pub(crate) fn rsa_verify(
    public_key: &rsa::RsaPublicKey,
    algorithm_name: &str,
    hash_name: &str,
    message: &[u8],
    signature: &[u8],
) -> std::result::Result<bool, &'static str> {
    use rsa::signature::Verifier;
    let result = match algorithm_name {
        "RSASSA-PKCS1-v1.5" => match hash_name {
            "SHA-1" => {
                let verifying_key =
                    rsa::pkcs1v15::VerifyingKey::<sha1::Sha1>::new(public_key.clone());
                let sig = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-256" => {
                let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha256>::new(public_key.clone());
                let sig = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-384" => {
                let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha384>::new(public_key.clone());
                let sig = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-512" => {
                let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha512>::new(public_key.clone());
                let sig = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            _ => return Err("unsupported hash algorithm"),
        },
        "RSA-PSS" => match hash_name {
            "SHA-1" => {
                let verifying_key = rsa::pss::VerifyingKey::<sha1::Sha1>::new(public_key.clone());
                let sig =
                    rsa::pss::Signature::try_from(signature).map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-256" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha256>::new(public_key.clone());
                let sig =
                    rsa::pss::Signature::try_from(signature).map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-384" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha384>::new(public_key.clone());
                let sig =
                    rsa::pss::Signature::try_from(signature).map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-512" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha512>::new(public_key.clone());
                let sig =
                    rsa::pss::Signature::try_from(signature).map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            _ => return Err("unsupported hash algorithm"),
        },
        _ => return Err("invalid RSA signing algorithm"),
    };
    Ok(result.is_ok())
}
