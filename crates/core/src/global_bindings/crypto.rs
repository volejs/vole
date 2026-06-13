use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Array, ArrayBuffer, Class, Constructor, Ctx, Function, IntoJs, Object, Promise, Result,
    TypedArray, Value,
};
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
    algorithm: String,
    usages: Vec<String>,
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

// =============================================================================
// Native classes
// =============================================================================

#[derive(Clone, Debug)]
pub struct Crypto;

unsafe impl<'js> rquickjs::JsLifetime<'js> for Crypto {
    type Changed<'to> = Crypto;
}

impl<'js> Trace<'js> for Crypto {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for Crypto {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for Crypto {
    const NAME: &'static str = "Crypto";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "getRandomValues",
            Function::new(ctx.clone(), crypto_get_random_values)?,
        )?;
        proto.set(
            "randomUUID",
            Function::new(ctx.clone(), crypto_random_uuid)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<Crypto> {
            Ok(Crypto)
        }
        let constr = Constructor::new_class::<Crypto, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

#[derive(Clone, Debug)]
pub struct SubtleCrypto;

unsafe impl<'js> rquickjs::JsLifetime<'js> for SubtleCrypto {
    type Changed<'to> = SubtleCrypto;
}

impl<'js> Trace<'js> for SubtleCrypto {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for SubtleCrypto {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for SubtleCrypto {
    const NAME: &'static str = "SubtleCrypto";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("digest", Function::new(ctx.clone(), crypto_subtle_digest)?)?;
        proto.set(
            "generateKey",
            Function::new(ctx.clone(), crypto_subtle_generate_key)?,
        )?;
        proto.set("encrypt", Function::new(ctx.clone(), crypto_subtle_encrypt)?)?;
        proto.set("decrypt", Function::new(ctx.clone(), crypto_subtle_decrypt)?)?;
        proto.set("sign", Function::new(ctx.clone(), crypto_subtle_sign)?)?;
        proto.set("verify", Function::new(ctx.clone(), crypto_subtle_verify)?)?;
        proto.set(
            "importKey",
            Function::new(ctx.clone(), crypto_subtle_import_key)?,
        )?;
        proto.set(
            "exportKey",
            Function::new(ctx.clone(), crypto_subtle_export_key)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<SubtleCrypto> {
            Ok(SubtleCrypto)
        }
        let constr = Constructor::new_class::<SubtleCrypto, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

#[derive(Clone, Debug)]
pub struct CryptoKey {
    id: u64,
    key_type: String,
    extractable: bool,
    algorithm: serde_json::Value,
    usages: Vec<String>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for CryptoKey {
    type Changed<'to> = CryptoKey;
}

impl<'js> Trace<'js> for CryptoKey {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for CryptoKey {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for CryptoKey {
    const NAME: &'static str = "CryptoKey";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("type", Function::new(ctx.clone(), crypto_key_type)?)?;
        proto.set(
            "extractable",
            Function::new(ctx.clone(), crypto_key_extractable)?,
        )?;
        proto.set(
            "algorithm",
            Function::new(ctx.clone(), crypto_key_algorithm)?,
        )?;
        proto.set("usages", Function::new(ctx.clone(), crypto_key_usages)?)?;
        proto.set("_id", Function::new(ctx.clone(), crypto_key_id)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<CryptoKey> {
            Err(rquickjs::Error::Exception)
        }
        let constr = Constructor::new_class::<CryptoKey, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

fn crypto_key_type<'js>(this: This<Class<'js, CryptoKey>>) -> Result<String> {
    Ok(this.try_borrow()?.key_type.clone())
}

fn crypto_key_extractable<'js>(this: This<Class<'js, CryptoKey>>) -> Result<bool> {
    Ok(this.try_borrow()?.extractable)
}

fn crypto_key_algorithm<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, CryptoKey>>,
) -> Result<Value<'js>> {
    let alg = this.try_borrow()?.algorithm.clone();
    json_value_to_js_value(&ctx, &alg)
}

fn crypto_key_usages<'js>(ctx: Ctx<'js>, this: This<Class<'js, CryptoKey>>) -> Result<Array<'js>> {
    let arr = Array::new(ctx.clone())?;
    for (i, u) in this.try_borrow()?.usages.iter().enumerate() {
        arr.set(i, u.clone())?;
    }
    Ok(arr)
}

fn crypto_key_id<'js>(this: This<Class<'js, CryptoKey>>) -> Result<u64> {
    Ok(this.try_borrow()?.id)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    Class::<Crypto>::define(&globals)?;
    Class::<SubtleCrypto>::define(&globals)?;
    Class::<CryptoKey>::define(&globals)?;

    // Turn the CryptoKey prototype helper methods into real getters.
    ctx.eval::<(), _>(
        r#"
        (function() {
            function wrapGetter(proto, name) {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: function() { return fn.call(this); },
                    enumerable: true,
                    configurable: true
                });
            }
            wrapGetter(CryptoKey.prototype, "type");
            wrapGetter(CryptoKey.prototype, "extractable");
            wrapGetter(CryptoKey.prototype, "algorithm");
            wrapGetter(CryptoKey.prototype, "usages");
            wrapGetter(CryptoKey.prototype, "_id");
        })();
    "#,
    )?;

    let subtle = Class::instance(ctx.clone(), SubtleCrypto)?;
    let crypto = Class::instance(ctx.clone(), Crypto)?;
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

// =============================================================================
// subtle.digest
// =============================================================================

fn crypto_subtle_digest<'js>(
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

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
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

// =============================================================================
// subtle.generateKey
// =============================================================================

fn crypto_subtle_generate_key<'js>(
    ctx: Ctx<'js>,
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

        let result: std::result::Result<Value<'js>, rquickjs::Error> = (|| {
            match name.as_str() {
                "AES-CBC" | "AES-GCM" => {
                    let length = algorithm_json["length"].as_u64().unwrap_or(256) as usize;
                    let key_len = length / 8;
                    let mut key_bytes = vec![0u8; key_len];
                    getrandom::getrandom(&mut key_bytes)
                        .map_err(|_| rquickjs::Error::Exception)?;
                    let id = store_key(KeyInfo {
                        material: KeyMaterial::Aes(key_bytes),
                        algorithm: name.clone(),
                        usages: usages_vec.clone(),
                        extractable,
                    });
                    let key = CryptoKey {
                        id,
                        key_type: "secret".to_string(),
                        extractable,
                        algorithm: algorithm_json.clone(),
                        usages: usages_vec.clone(),
                    };
                    Class::instance(ctx_clone.clone(), key).into_js(&ctx_clone)
                }
                "HMAC" => {
                    let hash = algorithm_json["hash"].as_str().unwrap_or("SHA-256");
                    let key_len = match hash {
                        "SHA-1" | "SHA-256" => 32,
                        "SHA-384" | "SHA-512" => 64,
                        _ => 32,
                    };
                    let mut key_bytes = vec![0u8; key_len];
                    getrandom::getrandom(&mut key_bytes)
                        .map_err(|_| rquickjs::Error::Exception)?;
                    let id = store_key(KeyInfo {
                        material: KeyMaterial::Hmac(key_bytes),
                        algorithm: format!("HMAC-{}", hash),
                        usages: usages_vec.clone(),
                        extractable,
                    });
                    let key = CryptoKey {
                        id,
                        key_type: "secret".to_string(),
                        extractable,
                        algorithm: algorithm_json.clone(),
                        usages: usages_vec.clone(),
                    };
                    Class::instance(ctx_clone.clone(), key).into_js(&ctx_clone)
                }
                "RSASSA-PKCS1-v1.5" | "RSA-PSS" => {
                    let bits = algorithm_json["modulusLength"]
                        .as_u64()
                        .unwrap_or(2048) as usize;
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
                        algorithm: name.clone(),
                        usages: usages_vec.clone(),
                        extractable,
                    });
                    let public_id = store_key(KeyInfo {
                        material: KeyMaterial::RsaPublic(public_der.as_bytes().to_vec()),
                        algorithm: name.clone(),
                        usages: filter_public_usages(&usages_vec),
                        extractable,
                    });

                    let private_key = CryptoKey {
                        id: private_id,
                        key_type: "private".to_string(),
                        extractable,
                        algorithm: algorithm_json.clone(),
                        usages: usages_vec.clone(),
                    };
                    let public_key = CryptoKey {
                        id: public_id,
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
                    pair.set(
                        "publicKey",
                        Class::instance(ctx_clone.clone(), public_key)?,
                    )?;
                    Ok(pair.into_value())
                }
                _ => Err(rquickjs::Error::Exception),
            }
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

// =============================================================================
// subtle.encrypt / subtle.decrypt
// =============================================================================

fn crypto_subtle_encrypt<'js>(
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

fn crypto_subtle_decrypt<'js>(
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

fn do_encrypt(
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

fn do_decrypt(
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

// =============================================================================
// subtle.sign / subtle.verify
// =============================================================================

fn crypto_subtle_sign<'js>(
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

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
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

fn crypto_subtle_verify<'js>(
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

    crate::global_bindings::timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                crate::global_bindings::timers::task_end();
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
            let private_key =
                rsa::RsaPrivateKey::from_pkcs8_der(der).map_err(|_| "invalid RSA private key")?;
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

// =============================================================================
// subtle.importKey / subtle.exportKey
// =============================================================================

fn crypto_subtle_import_key<'js>(
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

fn crypto_subtle_export_key<'js>(
    ctx: Ctx<'js>,
    format: String,
    key: Class<'js, CryptoKey>,
) -> Result<Promise<'js>> {
    let key_info = get_key(key.try_borrow()?.id).ok_or(rquickjs::Error::Exception)?;

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

fn import_jwk(jwk: &serde_json::Value, expected_name: &str) -> Result<KeyMaterial> {
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

// =============================================================================
// Helpers
// =============================================================================

fn value_to_bytes<'js>(value: &Value<'js>) -> Result<Vec<u8>> {
    value_as_bytes(value).ok_or(rquickjs::Error::Exception)
}

fn value_as_bytes<'js>(value: &Value<'js>) -> Option<Vec<u8>> {
    if let Ok(ta) = value.get::<TypedArray<'js, u8>>() {
        if let Some(bytes) = ta.as_bytes() {
            return Some(bytes.to_vec());
        }
    }
    if let Ok(ab) = value.get::<ArrayBuffer<'js>>() {
        return Some(ab.as_bytes().unwrap_or_default().to_vec());
    }
    if let Some(obj) = value.as_object() {
        if let Ok(buffer_val) = obj.get::<&str, Value>("buffer") {
            if let Ok(ab) = buffer_val.get::<ArrayBuffer<'js>>() {
                let offset = obj.get::<&str, i64>("byteOffset").unwrap_or(0).max(0) as usize;
                let length = obj.get::<&str, i64>("byteLength").unwrap_or(0).max(0) as usize;
                let bytes = ab.as_bytes().unwrap_or_default();
                if offset < bytes.len() {
                    let end = (offset + length).min(bytes.len());
                    return Some(bytes[offset..end].to_vec());
                }
            }
        }
    }
    None
}

fn js_value_to_json<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<serde_json::Value> {
    if value.is_undefined() || value.is_null() {
        return Ok(serde_json::Value::Null);
    }
    if let Some(b) = value.as_bool() {
        return Ok(serde_json::Value::Bool(b));
    }
    if let Some(n) = value.as_number() {
        let json_val = if n.is_finite() && n.fract() == 0.0 {
            if n >= 0.0 && n <= u64::MAX as f64 {
                serde_json::Value::Number((n as u64).into())
            } else if n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                serde_json::Value::Number((n as i64).into())
            } else {
                serde_json::Number::from_f64(n)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number)
            }
        } else {
            serde_json::Number::from_f64(n)
                .map_or(serde_json::Value::Null, serde_json::Value::Number)
        };
        return Ok(json_val);
    }
    if let Some(s) = value.as_string() {
        return Ok(serde_json::Value::String(s.to_string().ok().unwrap_or_default()));
    }
    if let Some(bytes) = value_as_bytes(value) {
        return Ok(serde_json::Value::Array(
            bytes.into_iter().map(|b| serde_json::Value::Number(b.into())).collect(),
        ));
    }
    if let Ok(arr) = value.get::<Array<'js>>() {
        let mut out = Vec::new();
        for v in arr.iter::<Value<'js>>() {
            out.push(js_value_to_json(ctx, &v?)?);
        }
        return Ok(serde_json::Value::Array(out));
    }
    if let Some(obj) = value.as_object() {
        let mut map = serde_json::Map::new();
        for entry in obj.props::<String, Value>() {
            let (k, v) = entry?;
            map.insert(k, js_value_to_json(ctx, &v)?);
        }
        return Ok(serde_json::Value::Object(map));
    }
    Ok(serde_json::Value::Null)
}

fn json_value_to_js_value<'js>(
    ctx: &Ctx<'js>,
    value: &serde_json::Value,
) -> Result<Value<'js>> {
    match value {
        serde_json::Value::Null => Ok(Value::new_null(ctx.clone())),
        serde_json::Value::Bool(b) => Ok(Value::new_bool(ctx.clone(), *b)),
        serde_json::Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            Ok(Value::new_number(ctx.clone(), f))
        }
        serde_json::Value::String(s) => Ok(s.clone().into_js(ctx)?),
        serde_json::Value::Array(arr) => {
            let a = Array::new(ctx.clone())?;
            for (i, v) in arr.iter().enumerate() {
                a.set(i, json_value_to_js_value(ctx, v)?)?;
            }
            Ok(a.into_value())
        }
        serde_json::Value::Object(map) => {
            let o = Object::new(ctx.clone())?;
            for (k, v) in map {
                o.set(k.as_str(), json_value_to_js_value(ctx, v)?)?;
            }
            Ok(o.into_value())
        }
    }
}
