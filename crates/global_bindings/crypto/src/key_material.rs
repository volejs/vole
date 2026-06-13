use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub(crate) enum KeyMaterial {
    Aes(Vec<u8>),
    Hmac(Vec<u8>),
    RsaPrivate(Vec<u8>),
    RsaPublic(Vec<u8>),
}

#[derive(Clone, Debug)]
pub(crate) struct KeyInfo {
    pub(crate) material: KeyMaterial,
    pub(crate) algorithm: String,
    pub(crate) usages: Vec<String>,
    pub(crate) extractable: bool,
}

static KEYS: std::sync::OnceLock<Mutex<HashMap<u64, KeyInfo>>> = std::sync::OnceLock::new();
static NEXT_KEY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn keys() -> &'static Mutex<HashMap<u64, KeyInfo>> {
    KEYS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn next_key_id() -> u64 {
    NEXT_KEY_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub(crate) fn filter_public_usages(usages: &[String]) -> Vec<String> {
    usages
        .iter()
        .filter(|u| u.as_str() == "verify" || u.as_str() == "encrypt" || u.as_str() == "wrapKey")
        .cloned()
        .collect()
}

pub(crate) fn store_key(info: KeyInfo) -> u64 {
    let id = next_key_id();
    keys().lock().unwrap().insert(id, info);
    id
}

pub(crate) fn get_key(id: u64) -> Option<KeyInfo> {
    keys().lock().unwrap().get(&id).cloned()
}
