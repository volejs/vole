// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! JS 字节源转换工具。
//!
//! 本模块统一处理 Web API 中常见的 `BufferSource` / 字节输入形态：
//! `ArrayBuffer`、TypedArray、`DataView`、JS 数组、字符串以及 Rust `Vec<u8>`。
//! Streams BYOB、Crypto、Blob、Fetch body、Encoding 等模块都应该复用这里的
//! 转换逻辑，避免每个 binding 自己实现一套不一致的 typed array 处理。
//!
//! 设计要点：
//! - [`ObjectBytes`] 保留原始 JS 对象引用，避免过早复制。
//! - 调用 [`ObjectBytes::as_bytes`] 可以借用底层字节，但需要处理 detached
//!   `ArrayBuffer` 的错误。
//! - 调用 [`ObjectBytes::into_bytes`] 会复制为 Rust `Vec<u8>`，适合需要脱离
//!   JS 生命周期的异步任务。
//! - 对字符串输入使用 JS 字符串转字节语义，不等同于 WebIDL 的 USVString
//!   规范化；需要严格 USVString 时应单独处理。

use std::rc::Rc;

use rquickjs::{
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::Constructor,
    ArrayBuffer, Coerced, Ctx, Error, Exception, FromJs, IntoJs, JsLifetime, Object, Result,
    TypedArray, Value,
};

use crate::result::ResultExt;

const ERROR_MSG_ARRAY_BUFFER_DETACHED: &str = "ArrayBuffer is detached";

#[derive(Clone, PartialEq)]
/// JS 字节输入的统一表示。
///
/// 注意：除 `Vec` 变体外，其余变体都持有 JS 对象。它们必须遵守 `'js`
/// 生命周期，不能跨 QuickJS context 或跨线程移动到不受 GC 跟踪的地方。
pub enum ObjectBytes<'js> {
    U8Array(TypedArray<'js, u8>),
    I8Array(TypedArray<'js, i8>),
    U16Array(TypedArray<'js, u16>),
    I16Array(TypedArray<'js, i16>),
    U32Array(TypedArray<'js, u32>),
    I32Array(TypedArray<'js, i32>),
    U64Array(TypedArray<'js, u64>),
    I64Array(TypedArray<'js, i64>),
    F32Array(TypedArray<'js, f32>),
    F64Array(TypedArray<'js, f64>),
    DataView(ArrayBuffer<'js>, usize, usize),
    Vec(Vec<u8>),
}

unsafe impl<'js> JsLifetime<'js> for ObjectBytes<'js> {
    type Changed<'to> = ObjectBytes<'to>;
}

impl<'js> Trace<'js> for ObjectBytes<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        match self {
            ObjectBytes::U8Array(a) => a.trace(tracer),
            ObjectBytes::I8Array(a) => a.trace(tracer),
            ObjectBytes::U16Array(a) => a.trace(tracer),
            ObjectBytes::I16Array(a) => a.trace(tracer),
            ObjectBytes::U32Array(a) => a.trace(tracer),
            ObjectBytes::I32Array(a) => a.trace(tracer),
            ObjectBytes::U64Array(a) => a.trace(tracer),
            ObjectBytes::I64Array(a) => a.trace(tracer),
            ObjectBytes::F32Array(a) => a.trace(tracer),
            ObjectBytes::F64Array(a) => a.trace(tracer),
            ObjectBytes::DataView(d, _, _) => d.trace(tracer),
            ObjectBytes::Vec(v) => v.trace(tracer),
        }
    }
}

impl<'js> IntoJs<'js> for ObjectBytes<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self {
            ObjectBytes::U8Array(a) => a.into_js(ctx),
            ObjectBytes::I8Array(a) => a.into_js(ctx),
            ObjectBytes::U16Array(a) => a.into_js(ctx),
            ObjectBytes::I16Array(a) => a.into_js(ctx),
            ObjectBytes::U32Array(a) => a.into_js(ctx),
            ObjectBytes::I32Array(a) => a.into_js(ctx),
            ObjectBytes::U64Array(a) => a.into_js(ctx),
            ObjectBytes::I64Array(a) => a.into_js(ctx),
            ObjectBytes::F32Array(a) => a.into_js(ctx),
            ObjectBytes::F64Array(a) => a.into_js(ctx),
            ObjectBytes::DataView(buffer, offset, length) => {
                let ctor: Constructor = ctx.globals().get(PredefinedAtom::DataView)?;
                ctor.construct((buffer, offset, length))
            },
            ObjectBytes::Vec(v) => v.into_js(ctx),
        }
    }
}

impl<'js> TryFrom<ObjectBytes<'js>> for Vec<u8> {
    type Error = Rc<str>;

    fn try_from(value: ObjectBytes<'js>) -> std::result::Result<Self, Self::Error> {
        value.into_bytes_inner()
    }
}

impl<'a, 'js> TryFrom<&'a ObjectBytes<'js>> for &'a [u8] {
    type Error = Rc<str>;

    fn try_from(value: &'a ObjectBytes<'js>) -> std::result::Result<Self, Self::Error> {
        value.as_bytes_inner()
    }
}

impl<'js> FromJs<'js> for ObjectBytes<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::from_offset(ctx, &value, 0, None)
    }
}

impl<'js> ObjectBytes<'js> {
    /// 从任意 JS 值中提取字节输入。
    ///
    /// 这个函数接受的范围比严格 WebIDL `BufferSource` 更宽，包括数组和
    /// 可强制转换为字符串的值。调用者如果需要严格标准行为，应在调用前
    /// 自行限制输入类型。
    pub fn from(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Self> {
        Self::from_offset(ctx, value, 0, None)
    }

    /// 从 JS 值中提取字节输入，并按 `offset` / `length` 裁剪。
    ///
    /// 对 `ArrayBufferView` 这类对象，裁剪语义由底层 view 的 byte range 决定；
    /// 对数组和字符串，则按元素/字节序列裁剪。
    pub fn from_offset(
        ctx: &Ctx<'js>,
        value: &Value<'js>,
        offset: usize,
        length: Option<usize>,
    ) -> Result<Self> {
        if value.is_undefined() {
            return Ok(ObjectBytes::Vec(vec![]));
        }
        if let Some(bytes) = get_string_bytes(value, offset, length)? {
            return Ok(ObjectBytes::Vec(bytes));
        }
        if let Some(bytes) = get_array_bytes(value, offset, length)? {
            return Ok(ObjectBytes::Vec(bytes));
        }
        if let Some(obj) = value.as_object() {
            if let Some(bytes) = Self::from_array_buffer(obj)? {
                return Ok(bytes);
            }
        }
        if let Some(bytes) = get_coerced_string_bytes(value, offset, length) {
            return Ok(ObjectBytes::Vec(bytes));
        }
        Err(Exception::throw_message(
            ctx,
            "value must be typed DataView, Buffer, ArrayBuffer, Uint8Array or interpretable as string",
        ))
    }

    /// 借用底层字节。
    ///
    /// 如果底层 `ArrayBuffer` 已 detached，会抛出 JS 异常。
    pub fn as_bytes(&self, ctx: &Ctx<'js>) -> Result<&[u8]> {
        self.as_bytes_inner().or_throw(ctx)
    }

    /// 尝试借用底层字节，detached 时返回 `None`。
    ///
    /// 适合 WebIDL 需要区分 detached buffer 和其他错误的场景。
    pub fn as_bytes_opt(&self) -> Option<&[u8]> {
        self.as_bytes_inner().ok()
    }

    fn as_bytes_inner(&self) -> std::result::Result<&[u8], Rc<str>> {
        match self {
            ObjectBytes::U8Array(array) => array.as_bytes(),
            ObjectBytes::I8Array(array) => array.as_bytes(),
            ObjectBytes::U16Array(array) => array.as_bytes(),
            ObjectBytes::I16Array(array) => array.as_bytes(),
            ObjectBytes::U32Array(array) => array.as_bytes(),
            ObjectBytes::I32Array(array) => array.as_bytes(),
            ObjectBytes::U64Array(array) => array.as_bytes(),
            ObjectBytes::I64Array(array) => array.as_bytes(),
            ObjectBytes::F32Array(array) => array.as_bytes(),
            ObjectBytes::F64Array(array) => array.as_bytes(),
            ObjectBytes::DataView(array_buffer, offset, length) => {
                array_buffer.as_bytes().map(|b| &b[*offset..*offset + *length])
            },
            ObjectBytes::Vec(bytes) => Some(bytes.as_ref()),
        }
        .ok_or(ERROR_MSG_ARRAY_BUFFER_DETACHED.into())
    }

    /// 复制为 Rust `Vec<u8>`。
    ///
    /// 如果后续逻辑会跨 async 边界或脱离 JS context 生命周期，应使用这个
    /// 方法，而不是保存 borrowed slice。
    pub fn into_bytes(self, ctx: &Ctx<'_>) -> Result<Vec<u8>> {
        self.into_bytes_inner().or_throw(ctx)
    }

    fn into_bytes_inner(self) -> std::result::Result<Vec<u8>, Rc<str>> {
        if let ObjectBytes::Vec(bytes) = self {
            return Ok(bytes);
        }
        Ok(self.as_bytes_inner()?.to_vec())
    }

    /// 判断一个 JS 对象是否是 `ArrayBuffer`、TypedArray 或 `DataView`。
    ///
    /// 返回的 `ObjectBytes` 仍然引用原始 JS 对象，不会复制底层字节。
    pub fn from_array_buffer(obj: &Object<'js>) -> Result<Option<ObjectBytes<'js>>> {
        if let Ok(typed_array) = TypedArray::<u8>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::U8Array(typed_array)));
        }
        if let Some(array_buffer) = ArrayBuffer::from_object(obj.clone()) {
            let len = array_buffer.len();
            return Ok(Some(ObjectBytes::DataView(array_buffer, 0, len)));
        }
        if let Ok(typed_array) = TypedArray::<i8>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::I8Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<u16>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::U16Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<i16>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::I16Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<u32>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::U32Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<i32>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::I32Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<u64>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::U64Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<i64>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::I64Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<f32>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::F32Array(typed_array)));
        }
        if let Ok(typed_array) = TypedArray::<f64>::from_object(obj.clone()) {
            return Ok(Some(ObjectBytes::F64Array(typed_array)));
        }
        if let Ok(array_buffer) = obj.get::<_, ArrayBuffer>("buffer") {
            let byte_offset: usize = obj.get("byteOffset").unwrap_or(0);
            let byte_length: usize = obj.get("byteLength").unwrap_or_else(|_| array_buffer.len());
            return Ok(Some(ObjectBytes::DataView(
                array_buffer,
                byte_offset,
                byte_length,
            )));
        }
        Ok(None)
    }

    /// 返回底层 `ArrayBuffer`、可见字节长度和 byteOffset。
    ///
    /// `Vec` 变体没有 JS `ArrayBuffer`，因此返回 `None`。
    pub fn get_array_buffer(&self) -> Result<Option<(ArrayBuffer<'js>, usize, usize)>> {
        let buffer = match self {
            ObjectBytes::DataView(array_buffer, offset, length) => {
                (array_buffer.clone(), *length, *offset)
            },
            ObjectBytes::U8Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len(), typed_array.get("byteOffset")?),
            ObjectBytes::I8Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len(), typed_array.get("byteOffset")?),
            ObjectBytes::U16Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 2, typed_array.get("byteOffset")?),
            ObjectBytes::I16Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 2, typed_array.get("byteOffset")?),
            ObjectBytes::U32Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 4, typed_array.get("byteOffset")?),
            ObjectBytes::I32Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 4, typed_array.get("byteOffset")?),
            ObjectBytes::U64Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 8, typed_array.get("byteOffset")?),
            ObjectBytes::I64Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 8, typed_array.get("byteOffset")?),
            ObjectBytes::F32Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 4, typed_array.get("byteOffset")?),
            ObjectBytes::F64Array(typed_array) => (typed_array.arraybuffer()?, typed_array.len() * 8, typed_array.get("byteOffset")?),
            ObjectBytes::Vec(_) => return Ok(None),
        };
        Ok(Some(buffer))
    }
}
pub fn get_start_end_indexes(
    source_len: usize,
    target_len: Option<usize>,
    offset: usize,
) -> (usize, usize) {
    if offset > source_len {
        return (0, 0);
    }
    let target_len = target_len.unwrap_or(source_len - offset);
    if offset + target_len > source_len {
        return (offset, source_len);
    }
    (offset, target_len + offset)
}

pub fn get_array_bytes(
    value: &Value<'_>,
    offset: usize,
    length: Option<usize>,
) -> Result<Option<Vec<u8>>> {
    if value.is_array() {
        let array = value.as_array().unwrap();
        let (start, end) = get_start_end_indexes(array.len(), length, offset);
        let size = end - start;
        let mut bytes = Vec::with_capacity(size);
        for val in array.iter::<u8>().skip(start).take(size) {
            bytes.push(val?);
        }
        return Ok(Some(bytes));
    }
    Ok(None)
}

pub fn get_coerced_string_bytes(
    value: &Value<'_>,
    offset: usize,
    length: Option<usize>,
) -> Option<Vec<u8>> {
    if let Ok(val) = value.get::<Coerced<String>>() {
        return Some(bytes_from_js_string(val.0, offset, length));
    }
    None
}

fn bytes_from_js_string(string: String, offset: usize, length: Option<usize>) -> Vec<u8> {
    let bytes = string.as_bytes();
    let (start, end) = get_start_end_indexes(bytes.len(), length, offset);
    bytes[start..end].to_vec()
}

pub fn get_string_bytes(
    value: &Value<'_>,
    offset: usize,
    length: Option<usize>,
) -> Result<Option<Vec<u8>>> {
    if value.is_string() {
        let string = value.clone().into_string().ok_or_else(|| Error::FromJs {
            from: "Value",
            to: "JSString",
            message: Some("Value is not a string".into()),
        })?;
        let string = string.to_string()?;
        return Ok(Some(bytes_from_js_string(string, offset, length)));
    }
    Ok(None)
}

pub fn bytes_to_typed_array<'js>(ctx: Ctx<'js>, bytes: &[u8]) -> Result<Value<'js>> {
    TypedArray::<u8>::new(ctx.clone(), bytes).into_js(&ctx)
}
