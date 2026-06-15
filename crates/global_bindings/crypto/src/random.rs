use rquickjs::{Ctx, Exception, Result, TypedArray, Value};

use crate::provider::{CryptoProvider, default_provider};
use vole_core::{
    error::{dom_exception_name, throw_dom_exception},
    webidl::{
        IntegerTypedArrayKind, ensure_integer_typed_array_max_byte_length, integer_typed_array_kind,
    },
};

pub(crate) fn crypto_random_uuid<'js>(ctx: Ctx<'js>) -> Result<String> {
    let mut bytes = [0u8; 16];
    fill_random_bytes(&ctx, &mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    ))
}

pub(crate) fn crypto_get_random_values<'js>(
    ctx: Ctx<'js>,
    array: Value<'js>,
) -> Result<Value<'js>> {
    ensure_integer_typed_array_max_byte_length(&ctx, &array, 65_536)?;
    let kind = integer_typed_array_kind(&array)?
        .ok_or_else(|| Exception::throw_type(&ctx, "Expected an integer TypedArray"))?;

    match kind {
        IntegerTypedArrayKind::Int8 => fill_typed_array(&ctx, array.get::<TypedArray<'js, i8>>()?)?,
        IntegerTypedArrayKind::Uint8 | IntegerTypedArrayKind::Uint8Clamped => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, u8>>()?)?
        }
        IntegerTypedArrayKind::Int16 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, i16>>()?)?
        }
        IntegerTypedArrayKind::Uint16 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, u16>>()?)?
        }
        IntegerTypedArrayKind::Int32 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, i32>>()?)?
        }
        IntegerTypedArrayKind::Uint32 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, u32>>()?)?
        }
        IntegerTypedArrayKind::BigInt64 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, i64>>()?)?
        }
        IntegerTypedArrayKind::BigUint64 => {
            fill_typed_array(&ctx, array.get::<TypedArray<'js, u64>>()?)?
        }
    }

    Ok(array)
}

fn fill_typed_array<'js, T>(ctx: &Ctx<'js>, array: TypedArray<'js, T>) -> Result<()> {
    let Some(raw) = array.as_raw() else {
        return Err(Exception::throw_type(ctx, "TypedArray buffer is detached"));
    };
    let byte_len = raw.len * std::mem::size_of::<T>();
    let bytes = unsafe { std::slice::from_raw_parts_mut(raw.ptr.as_ptr().cast::<u8>(), byte_len) };
    fill_random_bytes(ctx, bytes)
}

fn fill_random_bytes<'js>(ctx: &Ctx<'js>, bytes: &mut [u8]) -> Result<()> {
    default_provider().fill_random(bytes).map_err(|_| {
        throw_dom_exception::<()>(
            ctx,
            dom_exception_name::OPERATION,
            "The operation failed for an operation-specific reason",
        )
        .unwrap_err()
    })
}
