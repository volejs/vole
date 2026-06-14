//! WebIDL 参数转换辅助。
//!
//! `global_bindings/*` 中的大部分 API 都需要把 JS 值转换成 WebIDL 语义：
//! 字符串 coercion、`USVString`、`BufferSource`、整数 TypedArray 等。
//! 如果每个 binding 自己处理，错误类型、接受范围和 detached buffer 行为会
//! 很快分叉。本模块集中放这些“无业务语义”的转换规则。
//!
//! 注意事项：
//! - 这里只做参数转换，不实现具体 Web API 的业务状态机。
//! - 本模块尽量保留 JS coercion 过程中抛出的原始异常；只有类型不匹配时
//!   才主动抛 `TypeError` 或 DOMException。
//! - `USVString` 的 lone-surrogate 替换受 rquickjs 字符串转换能力限制。当前
//!   统一入口先集中到这里，后续如果需要更精确的 UTF-16 级处理，只改本模块。

use rquickjs::{Coerced, Ctx, Exception, Object, Result, TypedArray, Value};

use crate::{
    bytes::ObjectBytes,
    class::get_class_name,
    error::{dom_exception_name, throw_dom_exception},
};

/// WebIDL `DOMString` / JS `String()` coercion。
///
/// 该函数会调用 JS coercion，因此对象的 `toString` / `valueOf` 抛错时会保留
/// 原始 JS exception。
pub fn to_dom_string(value: &Value<'_>) -> Result<String> {
    value.get::<Coerced<String>>().map(|value| value.0)
}

/// WebIDL `USVString` coercion。
///
/// Rust `String` 不能保存 lone surrogate，因此 rquickjs 转换到 Rust 字符串时
/// 已经不再暴露原始 UTF-16 code unit。先把入口集中在这里，后续如需完全按
/// WebIDL 标准处理 surrogate，可在这里替换实现。
pub fn to_usv_string(value: &Value<'_>) -> Result<String> {
    to_dom_string(value)
}

/// 把 Web API 的 binary string 转成字节。
///
/// `btoa()` 等 API 使用 binary string 语义：每个字符的 code point 必须在
/// `0x00..=0xff`，对应一个字节。超出范围时应抛 `InvalidCharacterError`。
pub fn binary_string_to_bytes<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Vec<u8>> {
    let string = to_dom_string(value)?;
    let mut bytes = Vec::with_capacity(string.len());
    for ch in string.chars() {
        let code_point = ch as u32;
        if code_point > 0xff {
            return throw_dom_exception(
                ctx,
                dom_exception_name::INVALID_CHARACTER,
                "The string to be encoded contains characters outside of the Latin1 range",
            );
        }
        bytes.push(code_point as u8);
    }
    Ok(bytes)
}

/// 把字节转回 Web API binary string。
pub fn bytes_to_binary_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| char::from(*byte)).collect()
}

/// 严格按 WebIDL `BufferSource` 读取字节。
///
/// 与 [`ObjectBytes::from`] 不同，本函数只接受 `ArrayBuffer` 或
/// `ArrayBufferView`，不会接受数组、字符串或其它可 coercion 的值。
pub fn buffer_source_to_vec<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Vec<u8>> {
    let Some(object) = value.as_object() else {
        return Err(Exception::throw_type(
            ctx,
            "Expected ArrayBuffer or ArrayBufferView",
        ));
    };

    let Some(bytes) = ObjectBytes::from_array_buffer(object)? else {
        return Err(Exception::throw_type(
            ctx,
            "Expected ArrayBuffer or ArrayBufferView",
        ));
    };

    bytes.into_bytes(ctx)
}

/// WebCrypto `getRandomValues()` 允许的整数 TypedArray 类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerTypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    BigInt64,
    BigUint64,
}

impl IntegerTypedArrayKind {
    /// JS 构造器名称，仅用于错误消息或调试输出。
    pub fn constructor_name(self) -> &'static str {
        match self {
            Self::Int8 => "Int8Array",
            Self::Uint8 => "Uint8Array",
            Self::Uint8Clamped => "Uint8ClampedArray",
            Self::Int16 => "Int16Array",
            Self::Uint16 => "Uint16Array",
            Self::Int32 => "Int32Array",
            Self::Uint32 => "Uint32Array",
            Self::BigInt64 => "BigInt64Array",
            Self::BigUint64 => "BigUint64Array",
        }
    }
}

/// 判断 JS 值是否是 WebCrypto 认可的整数 TypedArray。
pub fn integer_typed_array_kind<'js>(
    value: &Value<'js>,
) -> Result<Option<IntegerTypedArrayKind>> {
    let Some(object) = value.as_object() else {
        return Ok(None);
    };

    if matches!(
        get_class_name(value)?.as_deref(),
        Some("Uint8ClampedArray")
    ) {
        return Ok(Some(IntegerTypedArrayKind::Uint8Clamped));
    }

    if TypedArray::<i8>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Int8));
    }
    if TypedArray::<u8>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Uint8));
    }
    if TypedArray::<i16>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Int16));
    }
    if TypedArray::<u16>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Uint16));
    }
    if TypedArray::<i32>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Int32));
    }
    if TypedArray::<u32>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::Uint32));
    }
    if TypedArray::<i64>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::BigInt64));
    }
    if TypedArray::<u64>::from_object(object.clone()).is_ok() {
        return Ok(Some(IntegerTypedArrayKind::BigUint64));
    }

    Ok(None)
}

/// 读取整数 TypedArray 的 `byteLength`。
pub fn integer_typed_array_byte_length<'js>(
    ctx: &Ctx<'js>,
    value: &Value<'js>,
) -> Result<usize> {
    if integer_typed_array_kind(value)?.is_none() {
        return Err(Exception::throw_type(ctx, "Expected an integer TypedArray"));
    }

    let object = value
        .as_object()
        .ok_or_else(|| Exception::throw_type(ctx, "Expected an integer TypedArray"))?;
    object.get("byteLength")
}

/// 确保整数 TypedArray 的字节长度不超过指定上限。
///
/// WebCrypto `getRandomValues()` 使用这个规则限制单次填充最多 65536 字节。
pub fn ensure_integer_typed_array_max_byte_length<'js>(
    ctx: &Ctx<'js>,
    value: &Value<'js>,
    max: usize,
) -> Result<usize> {
    let byte_length = integer_typed_array_byte_length(ctx, value)?;
    if byte_length > max {
        return throw_dom_exception(
            ctx,
            dom_exception_name::QUOTA_EXCEEDED,
            "The requested length exceeds the operation quota",
        );
    }
    Ok(byte_length)
}

/// 如果一个对象看起来像 ArrayBufferView，则读取其底层 byte range。
///
/// 该 helper 用于需要保留原始 `buffer`/`byteOffset`/`byteLength` 结构的 API。
pub fn array_buffer_view_range<'js>(
    object: &Object<'js>,
) -> Result<Option<(ObjectBytes<'js>, usize, usize)>> {
    let Some(bytes) = ObjectBytes::from_array_buffer(object)? else {
        return Ok(None);
    };

    let byte_offset = object.get("byteOffset").unwrap_or(0);
    let byte_length = object.get("byteLength").unwrap_or(0);
    Ok(Some((bytes, byte_offset, byte_length)))
}
