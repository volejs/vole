use rquickjs::{Array, ArrayBuffer, Ctx, Function, Object, Promise, Result, TypedArray, Value};
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha384, Sha512};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug)]
enum KeyMaterial {
    Aes(Vec<u8>),
    Hmac(Vec<u8>),
    RsaPrivate(Vec<u8>),
    RsaPublic(Vec<u8>),
}

#[derive(Clone, Debug)]
struct KeyInfo {
    material: KeyMaterial,
    #[allow(dead_code)]
    algorithm: String,
    #[allow(dead_code)]
    usages: Vec<String>,
    #[allow(dead_code)]
    extractable: bool,
}

static KEYS: std::sync::OnceLock<Mutex<HashMap<u64, KeyInfo>>> = std::sync::OnceLock::new();
static NEXT_KEY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn keys() -> &'static Mutex<HashMap<u64, KeyInfo>> {
    KEYS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_key_id() -> u64 {
    NEXT_KEY_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn filter_public_usages(usages: &[String]) -> Vec<String> {
    usages
        .iter()
        .filter(|u| u.as_str() == "verify" || u.as_str() == "encrypt" || u.as_str() == "wrapKey")
        .cloned()
        .collect()
}

fn store_key(info: KeyInfo) -> u64 {
    let id = next_key_id();
    keys().lock().unwrap().insert(id, info);
    id
}

fn get_key(id: u64) -> Option<KeyInfo> {
    keys().lock().unwrap().get(&id).cloned()
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    let crypto = Object::new(ctx.clone())?;
    crypto.set(
        "getRandomValues",
        Function::new(ctx.clone(), crypto_get_random_values)?,
    )?;
    crypto.set(
        "randomUUID",
        Function::new(ctx.clone(), crypto_random_uuid)?,
    )?;

    let subtle = Object::new(ctx.clone())?;
    subtle.set(
        "digest",
        Function::new(ctx.clone(), crypto_subtle_digest)?,
    )?;
    crypto.set("subtle", subtle)?;

    globals.set("crypto", crypto)?;

    Ok(())
}

fn crypto_random_uuid() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).unwrap();
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn crypto_get_random_values<'js>(array: TypedArray<'js, u8>) -> Result<TypedArray<'js, u8>> {
    if let Some(raw) = array.as_raw() {
        let slice = unsafe { std::slice::from_raw_parts_mut(raw.ptr.as_ptr(), raw.len) };
        getrandom::getrandom(slice).map_err(|_| rquickjs::Error::Exception)?;
    }
    Ok(array)
}

fn crypto_subtle_digest<'js>(
    ctx: Ctx<'js>,
    algorithm: String,
    data: Value<'js>,
) -> Result<Promise<'js>> {
    let bytes = value_to_bytes(&data)?;

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let result = match algorithm.as_str() {
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

        match TypedArray::new_copy(ctx_clone.clone(), result) {
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

// ===== generateKey =====

fn crypto_subtle_generate_key<'js>(
    ctx: Ctx<'js>,
    algorithm_json: String,
    extractable: bool,
    usages: Array<'js>,
) -> Result<Promise<'js>> {
    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;

    let usages_vec: Vec<String> = (0..usages.len())
        .map(|i| usages.get::<String>(i as usize).unwrap_or_default())
        .collect();

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let name = algorithm["name"].as_str().unwrap_or("");
        let result: std::result::Result<Object, rquickjs::Error> = (|| {
            let id = match name {
                "AES-CBC" | "AES-GCM" => {
                    let length = algorithm["length"].as_u64().unwrap_or(256) as usize;
                    let key_len = length / 8;
                    let mut key_bytes = vec![0u8; key_len];
                    getrandom::getrandom(&mut key_bytes).map_err(|_| rquickjs::Error::Exception)?;
                    store_key(KeyInfo {
                        material: KeyMaterial::Aes(key_bytes),
                        algorithm: name.to_string(),
                        usages: usages_vec.clone(),
                        extractable,
                    })
                }
                "HMAC" => {
                    let hash = algorithm["hash"].as_str().unwrap_or("SHA-256");
                    let key_len = match hash {
                        "SHA-1" | "SHA-256" => 32,
                        "SHA-384" | "SHA-512" => 64,
                        _ => 32,
                    };
                    let mut key_bytes = vec![0u8; key_len];
                    getrandom::getrandom(&mut key_bytes).map_err(|_| rquickjs::Error::Exception)?;
                    store_key(KeyInfo {
                        material: KeyMaterial::Hmac(key_bytes),
                        algorithm: format!("HMAC-{}", hash),
                        usages: usages_vec.clone(),
                        extractable,
                    })
                }
                "RSASSA-PKCS1-v1.5" | "RSA-PSS" => {
                    let bits = algorithm["modulusLength"].as_u64().unwrap_or(2048) as usize;
                    let private_key = rsa::RsaPrivateKey::new(&mut rand::thread_rng(), bits)
                        .map_err(|_| rquickjs::Error::Exception)?;
                    let public_key = rsa::RsaPublicKey::from(&private_key);
                    let private_der = private_key
                        .to_pkcs8_der()
                        .map_err(|_| rquickjs::Error::Exception)?;
                    let public_der = public_key
                        .to_public_key_der()
                        .map_err(|_| rquickjs::Error::Exception)?;
                    let private_id = store_key(KeyInfo {
                        material: KeyMaterial::RsaPrivate(private_der.as_bytes().to_vec()),
                        algorithm: name.to_string(),
                        usages: usages_vec.clone(),
                        extractable,
                    });
                    let public_id = store_key(KeyInfo {
                        material: KeyMaterial::RsaPublic(public_der.as_bytes().to_vec()),
                        algorithm: name.to_string(),
                        usages: filter_public_usages(&usages_vec),
                        extractable,
                    });
                    let pair = Object::new(ctx_clone.clone())?;
                    let private_obj = Object::new(ctx_clone.clone())?;
                    private_obj.set("_id", private_id)?;
                    private_obj.set("type", "private")?;
                    private_obj.set("extractable", extractable)?;
                    private_obj.set("algorithm", name)?;
                    private_obj.set("usages", usages_vec.clone())?;
                    let public_obj = Object::new(ctx_clone.clone())?;
                    public_obj.set("_id", public_id)?;
                    public_obj.set("type", "public")?;
                    public_obj.set("extractable", extractable)?;
                    public_obj.set("algorithm", name)?;
                    public_obj.set("usages", filter_public_usages(&usages_vec))?;
                    pair.set("privateKey", private_obj)?;
                    pair.set("publicKey", public_obj)?;
                    return Ok(pair);
                }
                _ => return Err(rquickjs::Error::Exception),
            };
            let obj = Object::new(ctx_clone.clone())?;
            obj.set("_id", id)?;
            obj.set("type", "secret")?;
            obj.set("extractable", extractable)?;
            obj.set("algorithm", name)?;
            obj.set("usages", usages_vec.clone())?;
            Ok(obj)
        })();

        match result {
            Ok(obj) => {
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(_) => {
                let _ = reject.call::<_, ()>(("generateKey failed",));
            }
        }
    });

    Ok(promise)
}

// ===== encrypt / decrypt =====

fn crypto_subtle_encrypt<'js>(
    ctx: Ctx<'js>,
    algorithm_json: String,
    key_id: u64,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let plaintext = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;
    let key_info = get_key(key_id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    ctx.spawn(async move {
        let result = do_encrypt(&key_info, &algorithm, &plaintext);
        match result {
            Ok(ciphertext) => {
                match TypedArray::new_copy(ctx_clone, ciphertext) {
                    Ok(ta) => { let _ = resolve.call::<_, ()>((ta,)); }
                    Err(_) => { let _ = reject.call::<_, ()>(("encrypt output failed",)); }
                }
            }
            Err(e) => { let _ = reject.call::<_, ()>((e,)); }
        }
    });

    Ok(promise)
}

fn crypto_subtle_decrypt<'js>(
    ctx: Ctx<'js>,
    algorithm_json: String,
    key_id: u64,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let ciphertext = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;
    let key_info = get_key(key_id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    ctx.spawn(async move {
        let result = do_decrypt(&key_info, &algorithm, &ciphertext);
        match result {
            Ok(plaintext) => {
                match TypedArray::new_copy(ctx_clone, plaintext) {
                    Ok(ta) => { let _ = resolve.call::<_, ()>((ta,)); }
                    Err(_) => { let _ = reject.call::<_, ()>(("decrypt output failed",)); }
                }
            }
            Err(e) => { let _ = reject.call::<_, ()>((e,)); }
        }
    });

    Ok(promise)
}

fn do_encrypt(key_info: &KeyInfo, algorithm: &serde_json::Value, plaintext: &[u8]) -> std::result::Result<Vec<u8>, &'static str> {
    match &key_info.material {
        KeyMaterial::Aes(key_bytes) => {
            let name = algorithm["name"].as_str().unwrap_or("");
            match name {
                "AES-CBC" => {
                    let iv = algorithm["iv"].as_array()
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
                    let iv = algorithm["iv"].as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 12 {
                        return Err("IV must be 12 bytes for GCM");
                    }
                    use aes_gcm::{Aes128Gcm, Aes256Gcm, aead::{Aead, KeyInit}};
                    let ciphertext = match key_bytes.len() {
                        16 => {
                            let key = aes_gcm::Key::<Aes128Gcm>::from_slice(key_bytes);
                            let cipher = Aes128Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher.encrypt(nonce, plaintext).map_err(|_| "Encrypt failed")?
                        }
                        32 => {
                            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
                            let cipher = Aes256Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher.encrypt(nonce, plaintext).map_err(|_| "Encrypt failed")?
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

fn do_decrypt(key_info: &KeyInfo, algorithm: &serde_json::Value, ciphertext: &[u8]) -> std::result::Result<Vec<u8>, &'static str> {
    match &key_info.material {
        KeyMaterial::Aes(key_bytes) => {
            let name = algorithm["name"].as_str().unwrap_or("");
            match name {
                "AES-CBC" => {
                    let iv = algorithm["iv"].as_array()
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
                    let iv = algorithm["iv"].as_array()
                        .ok_or("Missing iv")?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>();
                    if iv.len() != 12 {
                        return Err("IV must be 12 bytes for GCM");
                    }
                    use aes_gcm::{Aes128Gcm, Aes256Gcm, aead::{Aead, KeyInit}};
                    let plaintext = match key_bytes.len() {
                        16 => {
                            let key = aes_gcm::Key::<Aes128Gcm>::from_slice(key_bytes);
                            let cipher = Aes128Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher.decrypt(nonce, ciphertext).map_err(|_| "Decrypt failed")?
                        }
                        32 => {
                            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
                            let cipher = Aes256Gcm::new(key);
                            let nonce = aes_gcm::Nonce::from_slice(&iv);
                            cipher.decrypt(nonce, ciphertext).map_err(|_| "Decrypt failed")?
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

// ===== sign / verify =====

fn crypto_subtle_sign<'js>(
    ctx: Ctx<'js>,
    algorithm_json: String,
    key_id: u64,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let message = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;
    let key_info = get_key(key_id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let result = do_sign(&key_info, &algorithm, &message);
        match result {
            Ok(sig) => {
                match ArrayBuffer::new_copy(ctx_clone, sig) {
                    Ok(ab) => { let _ = resolve.call::<_, ()>((ab,)); }
                    Err(_) => { let _ = reject.call::<_, ()>(("sign output failed",)); }
                }
            }
            Err(e) => { let _ = reject.call::<_, ()>((e,)); }
        }
    });

    Ok(promise)
}

fn crypto_subtle_verify<'js>(
    ctx: Ctx<'js>,
    algorithm_json: String,
    key_id: u64,
    signature: TypedArray<'js, u8>,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let sig = if let Some(raw) = signature.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    let message = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;
    let key_info = get_key(key_id).ok_or(rquickjs::Error::Exception)?;

    let (promise, resolve, reject) = ctx.promise()?;

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
            }
        }
        let _guard = TaskGuard;

        let result = do_verify(&key_info, &algorithm, &message, &sig);
        match result {
            Ok(valid) => {
                let _ = resolve.call::<_, ()>((valid,));
            }
            Err(e) => { let _ = reject.call::<_, ()>((e,)); }
        }
    });

    Ok(promise)
}

fn hash_name(algorithm: &serde_json::Value) -> &'static str {
    match algorithm["hash"]["name"].as_str() {
        Some("SHA-1") => "SHA-1",
        Some("SHA-256") => "SHA-256",
        Some("SHA-384") => "SHA-384",
        Some("SHA-512") => "SHA-512",
        _ => "SHA-256",
    }
}

fn do_sign(
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
            let private_key = rsa::RsaPrivateKey::from_pkcs8_der(der)
                .map_err(|_| "invalid RSA private key")?;
            let hash_name = hash_name(algorithm);
            let alg_name = key_info.algorithm.as_str();
            rsa_sign(&private_key, alg_name, hash_name, message)
        }
        _ => Err("Invalid key type for signing"),
    }
}

fn do_verify(
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
            let private_key = rsa::RsaPrivateKey::from_pkcs8_der(der)
                .map_err(|_| "invalid RSA private key")?;
            let public_key = rsa::RsaPublicKey::from(&private_key);
            let hash_name = hash_name(algorithm);
            let alg_name = key_info.algorithm.as_str();
            rsa_verify(&public_key, alg_name, hash_name, message, signature)
        }
        _ => Err("Invalid key type for verification"),
    }
}

fn hmac_sign(
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

fn rsa_sign(
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
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-256" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha256>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-384" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha384>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-512" => {
                let signing_key = rsa::pkcs1v15::SigningKey::<Sha512>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            _ => Err("unsupported hash algorithm"),
        },
        "RSA-PSS" => match hash_name {
            "SHA-1" => {
                let signing_key = rsa::pss::SigningKey::<sha1::Sha1>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-256" => {
                let signing_key = rsa::pss::SigningKey::<Sha256>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-384" => {
                let signing_key = rsa::pss::SigningKey::<Sha384>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            "SHA-512" => {
                let signing_key = rsa::pss::SigningKey::<Sha512>::new(private_key.clone());
                Ok(signing_key.try_sign(message).map_err(|_| "sign failed")?.to_vec())
            }
            _ => Err("unsupported hash algorithm"),
        },
        _ => Err("invalid RSA signing algorithm"),
    }
}

fn rsa_verify(
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
                let verifying_key = rsa::pkcs1v15::VerifyingKey::<sha1::Sha1>::new(public_key.clone());
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
                let sig = rsa::pss::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-256" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha256>::new(public_key.clone());
                let sig = rsa::pss::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-384" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha384>::new(public_key.clone());
                let sig = rsa::pss::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            "SHA-512" => {
                let verifying_key = rsa::pss::VerifyingKey::<Sha512>::new(public_key.clone());
                let sig = rsa::pss::Signature::try_from(signature)
                    .map_err(|_| "invalid signature")?;
                verifying_key.verify(message, &sig)
            }
            _ => return Err("unsupported hash algorithm"),
        },
        _ => return Err("invalid RSA signing algorithm"),
    };
    Ok(result.is_ok())
}

// ===== importKey / exportKey =====

fn crypto_subtle_import_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key_data: Value<'js>,
    algorithm_json: String,
    extractable: bool,
    usages: Array<'js>,
) -> Result<Object<'js>> {
    let algorithm: serde_json::Value = serde_json::from_str(&algorithm_json)
        .map_err(|_| rquickjs::Error::Exception)?;
    let name = algorithm["name"].as_str().unwrap_or("");

    let usages_vec: Vec<String> = (0..usages.len())
        .map(|i| usages.get::<String>(i as usize).unwrap_or_default())
        .collect();

    let (material, key_type) = match format.as_str() {
        "raw" => {
            let bytes = value_to_bytes(&key_data)?;
            let material = match name {
                "AES-CBC" | "AES-GCM" => KeyMaterial::Aes(bytes),
                "HMAC" => KeyMaterial::Hmac(bytes),
                _ => return Err(rquickjs::Error::Exception),
            };
            (material, "secret")
        }
        "jwk" => {
            let jwk_str: String = key_data.get()?;
            let jwk: serde_json::Value = serde_json::from_str(&jwk_str)
                .map_err(|_| rquickjs::Error::Exception)?;
            let material = import_jwk(&jwk, name)?;
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
        algorithm: name.to_string(),
        usages: usages_vec.clone(),
        extractable,
    });

    let obj = Object::new(ctx)?;
    obj.set("_id", id)?;
    obj.set("type", key_type)?;
    obj.set("extractable", extractable)?;
    obj.set("algorithm", name)?;
    obj.set("usages", usages_vec)?;
    Ok(obj)
}

fn crypto_subtle_export_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key_id: u64,
) -> Result<Value<'js>> {
    let key_info = get_key(key_id).ok_or(rquickjs::Error::Exception)?;

    match format.as_str() {
        "raw" => {
            let bytes = match &key_info.material {
                KeyMaterial::Aes(b) => b.clone(),
                KeyMaterial::Hmac(b) => b.clone(),
                _ => return Err(rquickjs::Error::Exception),
            };
            Ok(ArrayBuffer::new_copy(ctx, bytes)?.into_value())
        }
        "jwk" => {
            let jwk = export_jwk(&key_info)?;
            let obj = Object::new(ctx)?;
            for (k, v) in jwk.as_object().ok_or(rquickjs::Error::Exception)? {
                obj.set(k, v.as_str().unwrap_or(""))?;
            }
            Ok(obj.into_value())
        }
        "pkcs8" => {
            let bytes = match &key_info.material {
                KeyMaterial::RsaPrivate(b) => b.clone(),
                _ => return Err(rquickjs::Error::Exception),
            };
            Ok(ArrayBuffer::new_copy(ctx, bytes)?.into_value())
        }
        "spki" => {
            let bytes = match &key_info.material {
                KeyMaterial::RsaPublic(b) => b.clone(),
                _ => return Err(rquickjs::Error::Exception),
            };
            Ok(ArrayBuffer::new_copy(ctx, bytes)?.into_value())
        }
        _ => Err(rquickjs::Error::Exception),
    }
}

// ===== JWK helpers =====

fn import_jwk(jwk: &serde_json::Value, expected_name: &str) -> Result<KeyMaterial> {
    let kty = jwk["kty"].as_str().ok_or(rquickjs::Error::Exception)?;
    if kty != "oct" {
        // Only symmetric (oct) keys are supported currently.
        return Err(rquickjs::Error::Exception);
    }

    let k = jwk["k"].as_str().ok_or(rquickjs::Error::Exception)?;
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(k)
        .map_err(|_| rquickjs::Error::Exception)?;

    // Validate algorithm if present.
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

fn export_jwk(key_info: &KeyInfo) -> Result<serde_json::Value> {
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
    jwk.insert("kty".to_string(), serde_json::Value::String("oct".to_string()));
    jwk.insert("k".to_string(), serde_json::Value::String(k));
    jwk.insert("alg".to_string(), serde_json::Value::String(alg.to_string()));
    jwk.insert("key_ops".to_string(), serde_json::Value::Array(
        key_info.usages.iter().map(|u| serde_json::Value::String(u.clone())).collect()
    ));
    jwk.insert("ext".to_string(), serde_json::Value::Bool(key_info.extractable));
    Ok(serde_json::Value::Object(jwk))
}

// ===== helpers =====

fn value_to_bytes<'js>(value: &Value<'js>) -> Result<Vec<u8>> {
    if let Ok(ta) = value.get::<TypedArray<'js, u8>>() {
        if let Some(raw) = ta.as_raw() {
            return Ok(unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec());
        }
    }
    if let Ok(ab) = value.get::<ArrayBuffer<'js>>() {
        return Ok(ab.as_bytes().unwrap_or_default().to_vec());
    }
    Err(rquickjs::Error::Exception)
}


pub struct CryptoRawModule;

impl ModuleDef for CryptoRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("__crypto_generate_key")?;
        decl.declare("__crypto_encrypt")?;
        decl.declare("__crypto_decrypt")?;
        decl.declare("__crypto_sign")?;
        decl.declare("__crypto_verify")?;
        decl.declare("__crypto_import_key")?;
        decl.declare("__crypto_export_key")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> rquickjs::Result<()> {
        exports.export("__crypto_generate_key", Function::new(ctx.clone(), crypto_subtle_generate_key))?;
        exports.export("__crypto_encrypt", Function::new(ctx.clone(), crypto_subtle_encrypt))?;
        exports.export("__crypto_decrypt", Function::new(ctx.clone(), crypto_subtle_decrypt))?;
        exports.export("__crypto_sign", Function::new(ctx.clone(), crypto_subtle_sign))?;
        exports.export("__crypto_verify", Function::new(ctx.clone(), crypto_subtle_verify))?;
        exports.export("__crypto_import_key", Function::new(ctx.clone(), crypto_subtle_import_key))?;
        exports.export("__crypto_export_key", Function::new(ctx.clone(), crypto_subtle_export_key))?;
        Ok(())
    }
}
