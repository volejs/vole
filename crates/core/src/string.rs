//! JS 字符串读取辅助。
//!
//! 这个模块只提供轻量的字符串读取和强制转换工具。更复杂的字节/字符串
//! 编码转换应使用 [`crate::encoding`]；需要按字节处理字符串时应使用
//! [`crate::bytes`]。
//!
//! 注意：这里的 `get_string` 读取 JS string 的 Rust 表示，不负责完整的
//! WebIDL USVString 标准化。如果某个 API 要求替换孤立 surrogate，应在
//! API 层明确处理。

use rquickjs::{Coerced, Exception, Result, Value};

#[inline]
/// 如果值本身是 JS string，则转换成 Rust `String`。
pub fn get_string(value: &Value<'_>) -> Result<Option<String>> {
    if let Some(val) = value.as_string() {
        let string = val.to_string()?;
        return Ok(Some(string));
    }
    Ok(None)
}

/// 尝试按 JS coercion 规则把值转换成字符串。
///
/// 转换失败时返回 `None`，不会抛出 JS exception。
pub fn get_coerced_string(value: &Value<'_>) -> Option<String> {
    if let Ok(val) = value.get::<Coerced<String>>() {
        return Some(val.0);
    };
    None
}

/// 读取 JS string，并返回 Rust `String`。
///
/// 这个 helper 用于承接 LLRT 中的 `get_lossy_string` 调用点。当前 rquickjs
/// 暴露的是 `JsString::to_string()`，因此这里主要提供统一入口；如果未来需要
/// 严格实现 WebIDL `USVString` 的 lone-surrogate 替换语义，应在这里集中增强。
pub fn get_lossy_string(value: Value<'_>) -> Result<String> {
    if let Some(string) = value.as_string() {
        return string.to_string();
    }

    Err(Exception::throw_type(
        value.ctx(),
        "value must be a string",
    ))
}
