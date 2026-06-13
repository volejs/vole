use rquickjs::{ArrayBuffer, Class, Ctx, Promise, Result, Value};

use crate::classes::CryptoKey;
use crate::key_material::{get_key, KeyInfo, KeyMaterial};
use crate::utils::{js_value_to_json, value_to_bytes};

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
    let plaintext = value_to_bytes(&data)?;
    let key_info = get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
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
    let ciphertext = value_to_bytes(&data)?;
    let key_info = get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
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
                    let iv = algorithm["iv"]
                        .as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 16 {
                        return Err("IV must be 16 bytes");
                    }
                    let iv_arr: &[u8; 16] = iv.as_slice().try_into().map_err(|_| "Invalid IV")?;
                    let key_arr: &[u8] = key_bytes.as_slice();
                    let ciphertext = match key_bytes.len() {
                        16 => {
                            use aes::Aes128;
                            use cbc::Encryptor;
                            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
                            Encryptor::<Aes128>::new(key_arr.into(), iv_arr.into())
                                .encrypt_padded_vec_mut::<Pkcs7>(plaintext)
                        }
                        24 => {
                            use aes::Aes192;
                            use cbc::Encryptor;
                            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
                            Encryptor::<Aes192>::new(key_arr.into(), iv_arr.into())
                                .encrypt_padded_vec_mut::<Pkcs7>(plaintext)
                        }
                        32 => {
                            use aes::Aes256;
                            use cbc::Encryptor;
                            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
                            Encryptor::<Aes256>::new(key_arr.into(), iv_arr.into())
                                .encrypt_padded_vec_mut::<Pkcs7>(plaintext)
                        }
                        _ => return Err("Invalid AES key length"),
                    };
                    Ok(ciphertext)
                }
                "AES-GCM" => {
                    let iv = algorithm["iv"]
                        .as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 12 {
                        return Err("IV must be 12 bytes for GCM");
                    }
                    use aes_gcm::{
                        aead::{Aead, KeyInit},
                        Aes128Gcm, Aes256Gcm,
                    };
                    let ciphertext = match key_bytes.len() {
                        16 => {
                            let key = aes_gcm::Key::<Aes128Gcm>::from_slice(key_bytes);
                            let cipher = Aes128Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher
                                .encrypt(nonce, plaintext)
                                .map_err(|_| "Encrypt failed")?
                        }
                        32 => {
                            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
                            let cipher = Aes256Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher
                                .encrypt(nonce, plaintext)
                                .map_err(|_| "Encrypt failed")?
                        }
                        _ => return Err("Invalid AES-GCM key length"),
                    };
                    Ok(ciphertext)
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
                    let iv = algorithm["iv"]
                        .as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 16 {
                        return Err("IV must be 16 bytes");
                    }
                    let iv_arr: &[u8; 16] = iv.as_slice().try_into().map_err(|_| "Invalid IV")?;
                    let key_arr: &[u8] = key_bytes.as_slice();
                    let plaintext = match key_bytes.len() {
                        16 => {
                            use aes::Aes128;
                            use cbc::Decryptor;
                            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
                            Decryptor::<Aes128>::new(key_arr.into(), iv_arr.into())
                                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                                .map_err(|_| "Decrypt failed")?
                        }
                        24 => {
                            use aes::Aes192;
                            use cbc::Decryptor;
                            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
                            Decryptor::<Aes192>::new(key_arr.into(), iv_arr.into())
                                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                                .map_err(|_| "Decrypt failed")?
                        }
                        32 => {
                            use aes::Aes256;
                            use cbc::Decryptor;
                            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
                            Decryptor::<Aes256>::new(key_arr.into(), iv_arr.into())
                                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                                .map_err(|_| "Decrypt failed")?
                        }
                        _ => return Err("Invalid AES key length"),
                    };
                    Ok(plaintext)
                }
                "AES-GCM" => {
                    let iv = algorithm["iv"]
                        .as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 12 {
                        return Err("IV must be 12 bytes for GCM");
                    }
                    use aes_gcm::{
                        aead::{Aead, KeyInit},
                        Aes128Gcm, Aes256Gcm,
                    };
                    let plaintext = match key_bytes.len() {
                        16 => {
                            let key = aes_gcm::Key::<Aes128Gcm>::from_slice(key_bytes);
                            let cipher = Aes128Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher
                                .decrypt(nonce, ciphertext)
                                .map_err(|_| "Decrypt failed")?
                        }
                        32 => {
                            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
                            let cipher = Aes256Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher
                                .decrypt(nonce, ciphertext)
                                .map_err(|_| "Decrypt failed")?
                        }
                        _ => return Err("Invalid AES-GCM key length"),
                    };
                    Ok(plaintext)
                }
                _ => Err("Unsupported AES mode"),
            }
        }
        _ => Err("Invalid key type for decryption"),
    }
}
