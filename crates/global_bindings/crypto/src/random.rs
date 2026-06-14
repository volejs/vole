use rquickjs::{Result, TypedArray};

pub(crate) fn crypto_random_uuid() -> String {
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

pub(crate) fn crypto_get_random_values<'js>(
    array: TypedArray<'js, u8>,
) -> Result<TypedArray<'js, u8>> {
    if let Some(raw) = array.as_raw() {
        let slice = unsafe { std::slice::from_raw_parts_mut(raw.ptr.as_ptr(), raw.len) };
        getrandom::getrandom(slice).map_err(|_| rquickjs::Error::Exception)?;
    }
    Ok(array)
}
