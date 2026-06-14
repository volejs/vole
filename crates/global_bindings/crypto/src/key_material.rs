#[derive(Clone, Debug)]
pub(crate) enum KeyMaterial {
    Aes(Vec<u8>),
    Hmac(Vec<u8>),
    Derive(Vec<u8>),
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

pub(crate) fn filter_public_usages(usages: &[String]) -> Vec<String> {
    usages
        .iter()
        .filter(|u| u.as_str() == "verify" || u.as_str() == "encrypt" || u.as_str() == "wrapKey")
        .cloned()
        .collect()
}

pub(crate) fn has_usage(info: &KeyInfo, usage: &str) -> bool {
    info.usages.iter().any(|candidate| candidate == usage)
}
