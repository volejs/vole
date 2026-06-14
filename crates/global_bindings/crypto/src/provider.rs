use aes::cipher::{
    generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit as BlockKeyInit,
};
use aes_gcm::{
    aead::{Aead, Payload},
    Aes128Gcm, Aes256Gcm,
};
use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha384, Sha512};

/// Crypto 后端抽象。
///
/// WebCrypto 绑定层只负责 WebIDL 参数转换、Promise 生命周期和 DOMException
/// 语义；具体算法实现应尽量通过这个 provider 边界进入。这样后续补齐
/// AES-CTR/AES-KW/ECDH/HKDF/PBKDF2，或者切换到 ring/OpenSSL 后端时，不需要
/// 重写 JS 暴露层。
pub(crate) trait CryptoProvider {
    fn fill_random(&self, bytes: &mut [u8]) -> std::result::Result<(), CryptoError>;
    fn digest(&self, algorithm: HashAlgorithm, data: &[u8]) -> Vec<u8>;
    fn hmac_sign(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        data: &[u8],
    ) -> std::result::Result<Vec<u8>, CryptoError>;
    fn aes_encrypt(
        &self,
        mode: AesMode,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        additional_data: Option<&[u8]>,
    ) -> std::result::Result<Vec<u8>, CryptoError>;
    fn aes_decrypt(
        &self,
        mode: AesMode,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        additional_data: Option<&[u8]>,
    ) -> std::result::Result<Vec<u8>, CryptoError>;
    fn aes_kw_wrap(&self, kek: &[u8], key: &[u8]) -> std::result::Result<Vec<u8>, CryptoError>;
    fn aes_kw_unwrap(
        &self,
        kek: &[u8],
        wrapped_key: &[u8],
    ) -> std::result::Result<Vec<u8>, CryptoError>;
    fn hkdf_derive(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        salt: &[u8],
        info: &[u8],
        length: usize,
    ) -> std::result::Result<Vec<u8>, CryptoError>;
    fn pbkdf2_derive(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        salt: &[u8],
        iterations: u32,
        length: usize,
    ) -> std::result::Result<Vec<u8>, CryptoError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CryptoError {
    InvalidLength,
    OperationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithm {
    pub(crate) fn from_web_name(name: &str) -> Option<Self> {
        match name {
            "SHA-1" => Some(Self::Sha1),
            "SHA-256" => Some(Self::Sha256),
            "SHA-384" => Some(Self::Sha384),
            "SHA-512" => Some(Self::Sha512),
            _ => None,
        }
    }

    pub(crate) fn digest_len(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AesMode {
    Cbc,
    Ctr { counter_length: u32 },
    Gcm,
}

#[derive(Debug)]
pub(crate) struct RustCryptoProvider;

static DEFAULT_PROVIDER: RustCryptoProvider = RustCryptoProvider;

pub(crate) fn default_provider() -> &'static RustCryptoProvider {
    &DEFAULT_PROVIDER
}

impl CryptoProvider for RustCryptoProvider {
    fn fill_random(&self, bytes: &mut [u8]) -> std::result::Result<(), CryptoError> {
        getrandom::getrandom(bytes).map_err(|_| CryptoError::OperationFailed)
    }

    fn digest(&self, algorithm: HashAlgorithm, data: &[u8]) -> Vec<u8> {
        match algorithm {
            HashAlgorithm::Sha1 => {
                let mut hasher = Sha1::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            },
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            },
            HashAlgorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            },
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            },
        }
    }

    fn hmac_sign(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        data: &[u8],
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        match algorithm {
            HashAlgorithm::Sha1 => {
                type HmacSha = Hmac<Sha1>;
                let mut mac =
                    <HmacSha as Mac>::new_from_slice(key).map_err(|_| CryptoError::InvalidLength)?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            },
            HashAlgorithm::Sha256 => {
                type HmacSha = Hmac<Sha256>;
                let mut mac =
                    <HmacSha as Mac>::new_from_slice(key).map_err(|_| CryptoError::InvalidLength)?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            },
            HashAlgorithm::Sha384 => {
                type HmacSha = Hmac<Sha384>;
                let mut mac =
                    <HmacSha as Mac>::new_from_slice(key).map_err(|_| CryptoError::InvalidLength)?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            },
            HashAlgorithm::Sha512 => {
                type HmacSha = Hmac<Sha512>;
                let mut mac =
                    <HmacSha as Mac>::new_from_slice(key).map_err(|_| CryptoError::InvalidLength)?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            },
        }
    }

    fn aes_encrypt(
        &self,
        mode: AesMode,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        additional_data: Option<&[u8]>,
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        match mode {
            AesMode::Cbc => aes_cbc_encrypt(key, iv, data),
            AesMode::Ctr { counter_length } => aes_ctr_xor(key, iv, counter_length, data),
            AesMode::Gcm => aes_gcm_encrypt(key, iv, data, additional_data.unwrap_or_default()),
        }
    }

    fn aes_decrypt(
        &self,
        mode: AesMode,
        key: &[u8],
        iv: &[u8],
        data: &[u8],
        additional_data: Option<&[u8]>,
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        match mode {
            AesMode::Cbc => aes_cbc_decrypt(key, iv, data),
            AesMode::Ctr { counter_length } => aes_ctr_xor(key, iv, counter_length, data),
            AesMode::Gcm => aes_gcm_decrypt(key, iv, data, additional_data.unwrap_or_default()),
        }
    }

    fn aes_kw_wrap(&self, kek: &[u8], key: &[u8]) -> std::result::Result<Vec<u8>, CryptoError> {
        match kek.len() {
            16 => aes_kw_wrap_with::<aes::Aes128>(kek, key),
            24 => aes_kw_wrap_with::<aes::Aes192>(kek, key),
            32 => aes_kw_wrap_with::<aes::Aes256>(kek, key),
            _ => Err(CryptoError::InvalidLength),
        }
    }

    fn aes_kw_unwrap(
        &self,
        kek: &[u8],
        wrapped_key: &[u8],
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        match kek.len() {
            16 => aes_kw_unwrap_with::<aes::Aes128>(kek, wrapped_key),
            24 => aes_kw_unwrap_with::<aes::Aes192>(kek, wrapped_key),
            32 => aes_kw_unwrap_with::<aes::Aes256>(kek, wrapped_key),
            _ => Err(CryptoError::InvalidLength),
        }
    }

    fn hkdf_derive(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        salt: &[u8],
        info: &[u8],
        length: usize,
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        hkdf_derive(self, algorithm, key, salt, info, length)
    }

    fn pbkdf2_derive(
        &self,
        algorithm: HashAlgorithm,
        key: &[u8],
        salt: &[u8],
        iterations: u32,
        length: usize,
    ) -> std::result::Result<Vec<u8>, CryptoError> {
        pbkdf2_derive(self, algorithm, key, salt, iterations, length)
    }
}

fn aes_cbc_encrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError> {
    if iv.len() != 16 {
        return Err(CryptoError::InvalidLength);
    }
    match key.len() {
        16 => {
            use aes::Aes128;
            use cbc::Encryptor;
            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
            Ok(Encryptor::<Aes128>::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data))
        },
        24 => {
            use aes::Aes192;
            use cbc::Encryptor;
            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
            Ok(Encryptor::<Aes192>::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data))
        },
        32 => {
            use aes::Aes256;
            use cbc::Encryptor;
            use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
            Ok(Encryptor::<Aes256>::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data))
        },
        _ => Err(CryptoError::InvalidLength),
    }
}

fn aes_cbc_decrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError> {
    if iv.len() != 16 {
        return Err(CryptoError::InvalidLength);
    }
    match key.len() {
        16 => {
            use aes::Aes128;
            use cbc::Decryptor;
            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
            Decryptor::<Aes128>::new(key.into(), iv.into())
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|_| CryptoError::OperationFailed)
        },
        24 => {
            use aes::Aes192;
            use cbc::Decryptor;
            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
            Decryptor::<Aes192>::new(key.into(), iv.into())
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|_| CryptoError::OperationFailed)
        },
        32 => {
            use aes::Aes256;
            use cbc::Decryptor;
            use cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
            Decryptor::<Aes256>::new(key.into(), iv.into())
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|_| CryptoError::OperationFailed)
        },
        _ => Err(CryptoError::InvalidLength),
    }
}

fn aes_gcm_encrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    additional_data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError> {
    if iv.len() != 12 {
        return Err(CryptoError::InvalidLength);
    }
    match key.len() {
        16 => {
            let cipher = Aes128Gcm::new(aes_gcm::Key::<Aes128Gcm>::from_slice(key));
            cipher
                .encrypt(aes_gcm::Nonce::from_slice(iv), Payload { msg: data, aad: additional_data })
                .map_err(|_| CryptoError::OperationFailed)
        },
        32 => {
            let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(key));
            cipher
                .encrypt(aes_gcm::Nonce::from_slice(iv), Payload { msg: data, aad: additional_data })
                .map_err(|_| CryptoError::OperationFailed)
        },
        _ => Err(CryptoError::InvalidLength),
    }
}

fn aes_gcm_decrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    additional_data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError> {
    if iv.len() != 12 {
        return Err(CryptoError::InvalidLength);
    }
    match key.len() {
        16 => {
            let cipher = Aes128Gcm::new(aes_gcm::Key::<Aes128Gcm>::from_slice(key));
            cipher
                .decrypt(aes_gcm::Nonce::from_slice(iv), Payload { msg: data, aad: additional_data })
                .map_err(|_| CryptoError::OperationFailed)
        },
        32 => {
            let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(key));
            cipher
                .decrypt(aes_gcm::Nonce::from_slice(iv), Payload { msg: data, aad: additional_data })
                .map_err(|_| CryptoError::OperationFailed)
        },
        _ => Err(CryptoError::InvalidLength),
    }
}

fn aes_ctr_xor(
    key: &[u8],
    counter: &[u8],
    counter_length: u32,
    data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError> {
    match key.len() {
        16 => aes_ctr_xor_with::<aes::Aes128>(key, counter, counter_length, data),
        24 => aes_ctr_xor_with::<aes::Aes192>(key, counter, counter_length, data),
        32 => aes_ctr_xor_with::<aes::Aes256>(key, counter, counter_length, data),
        _ => Err(CryptoError::InvalidLength),
    }
}

fn aes_ctr_xor_with<C>(
    key: &[u8],
    counter: &[u8],
    counter_length: u32,
    data: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError>
where
    C: BlockEncrypt + BlockKeyInit,
{
    if counter.len() != 16 || !(1..=128).contains(&counter_length) {
        return Err(CryptoError::InvalidLength);
    }

    let cipher = C::new_from_slice(key).map_err(|_| CryptoError::InvalidLength)?;
    let mut counter_block = [0u8; 16];
    counter_block.copy_from_slice(counter);

    let mut output = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        let mut keystream = GenericArray::clone_from_slice(&counter_block);
        cipher.encrypt_block(&mut keystream);
        output.extend(chunk.iter().zip(keystream.iter()).map(|(byte, key)| *byte ^ *key));
        increment_counter_bits(&mut counter_block, counter_length);
    }

    Ok(output)
}

fn increment_counter_bits(counter: &mut [u8; 16], counter_length: u32) {
    for bit_index in 0..counter_length {
        let byte_index = 15 - (bit_index / 8) as usize;
        let bit_mask = 1u8 << (bit_index % 8);
        if counter[byte_index] & bit_mask == 0 {
            counter[byte_index] |= bit_mask;
            break;
        }
        counter[byte_index] &= !bit_mask;
    }
}

fn aes_kw_wrap_with<C>(kek: &[u8], key: &[u8]) -> std::result::Result<Vec<u8>, CryptoError>
where
    C: BlockEncrypt + BlockKeyInit,
{
    if key.len() < 16 || key.len() % 8 != 0 {
        return Err(CryptoError::InvalidLength);
    }

    let cipher = C::new_from_slice(kek).map_err(|_| CryptoError::InvalidLength)?;
    let n = key.len() / 8;
    let mut a = [0xa6u8; 8];
    let mut r = key
        .chunks_exact(8)
        .map(|chunk| {
            let mut block = [0u8; 8];
            block.copy_from_slice(chunk);
            block
        })
        .collect::<Vec<_>>();

    for j in 0..6 {
        for (i, block) in r.iter_mut().enumerate() {
            let mut b = [0u8; 16];
            b[..8].copy_from_slice(&a);
            b[8..].copy_from_slice(block);
            let mut encrypted = GenericArray::clone_from_slice(&b);
            cipher.encrypt_block(&mut encrypted);
            a.copy_from_slice(&encrypted[..8]);
            xor_u64_be(&mut a, (n * j + i + 1) as u64);
            block.copy_from_slice(&encrypted[8..]);
        }
    }

    let mut output = Vec::with_capacity(key.len() + 8);
    output.extend_from_slice(&a);
    for block in r {
        output.extend_from_slice(&block);
    }
    Ok(output)
}

fn aes_kw_unwrap_with<C>(
    kek: &[u8],
    wrapped_key: &[u8],
) -> std::result::Result<Vec<u8>, CryptoError>
where
    C: BlockDecrypt + BlockKeyInit,
{
    if wrapped_key.len() < 24 || wrapped_key.len() % 8 != 0 {
        return Err(CryptoError::InvalidLength);
    }

    let cipher = C::new_from_slice(kek).map_err(|_| CryptoError::InvalidLength)?;
    let n = wrapped_key.len() / 8 - 1;
    let mut a = [0u8; 8];
    a.copy_from_slice(&wrapped_key[..8]);
    let mut r = wrapped_key[8..]
        .chunks_exact(8)
        .map(|chunk| {
            let mut block = [0u8; 8];
            block.copy_from_slice(chunk);
            block
        })
        .collect::<Vec<_>>();

    for j in (0..6).rev() {
        for i in (0..n).rev() {
            let mut a_xor_t = a;
            xor_u64_be(&mut a_xor_t, (n * j + i + 1) as u64);
            let mut b = [0u8; 16];
            b[..8].copy_from_slice(&a_xor_t);
            b[8..].copy_from_slice(&r[i]);
            let mut decrypted = GenericArray::clone_from_slice(&b);
            cipher.decrypt_block(&mut decrypted);
            a.copy_from_slice(&decrypted[..8]);
            r[i].copy_from_slice(&decrypted[8..]);
        }
    }

    if a != [0xa6u8; 8] {
        return Err(CryptoError::OperationFailed);
    }

    let mut output = Vec::with_capacity(wrapped_key.len() - 8);
    for block in r {
        output.extend_from_slice(&block);
    }
    Ok(output)
}

fn xor_u64_be(bytes: &mut [u8; 8], value: u64) {
    let value = value.to_be_bytes();
    for (target, source) in bytes.iter_mut().zip(value) {
        *target ^= source;
    }
}

fn hkdf_derive<P: CryptoProvider>(
    provider: &P,
    algorithm: HashAlgorithm,
    key: &[u8],
    salt: &[u8],
    info: &[u8],
    length: usize,
) -> std::result::Result<Vec<u8>, CryptoError> {
    let hash_len = algorithm.digest_len();
    if length > 255 * hash_len {
        return Err(CryptoError::InvalidLength);
    }

    let prk = provider.hmac_sign(algorithm, salt, key)?;
    let mut output = Vec::with_capacity(length);
    let mut previous = Vec::new();
    let mut counter = 1u8;

    while output.len() < length {
        let mut input = Vec::with_capacity(previous.len() + info.len() + 1);
        input.extend_from_slice(&previous);
        input.extend_from_slice(info);
        input.push(counter);
        previous = provider.hmac_sign(algorithm, &prk, &input)?;
        output.extend_from_slice(&previous);
        counter = counter.checked_add(1).ok_or(CryptoError::InvalidLength)?;
    }

    output.truncate(length);
    Ok(output)
}

fn pbkdf2_derive<P: CryptoProvider>(
    provider: &P,
    algorithm: HashAlgorithm,
    key: &[u8],
    salt: &[u8],
    iterations: u32,
    length: usize,
) -> std::result::Result<Vec<u8>, CryptoError> {
    if iterations == 0 {
        return Err(CryptoError::InvalidLength);
    }

    let hash_len = algorithm.digest_len();
    let block_count = length.div_ceil(hash_len);
    if block_count > u32::MAX as usize {
        return Err(CryptoError::InvalidLength);
    }

    let mut output = Vec::with_capacity(block_count * hash_len);
    for block_index in 1..=block_count {
        let mut first_input = Vec::with_capacity(salt.len() + 4);
        first_input.extend_from_slice(salt);
        first_input.extend_from_slice(&(block_index as u32).to_be_bytes());

        let mut u = provider.hmac_sign(algorithm, key, &first_input)?;
        let mut t = u.clone();
        for _ in 1..iterations {
            u = provider.hmac_sign(algorithm, key, &u)?;
            for (target, source) in t.iter_mut().zip(u.iter()) {
                *target ^= *source;
            }
        }
        output.extend_from_slice(&t);
    }

    output.truncate(length);
    Ok(output)
}
