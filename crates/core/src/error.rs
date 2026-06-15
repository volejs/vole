// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! rquickjs 错误到 JS 值的转换工具。
//!
//! native binding 经常需要把 Rust 层捕获到的 `rquickjs::Error` 或
//! `CaughtError` 变成 JS 值，用来 reject Promise、派发 error event 或
//! 存储为 stream 的 stored error。
//!
//! 注意事项：
//! - 本模块只做“错误对象转换”，不负责错误分发策略。
//! - 是否派发 `error` / `unhandledrejection` 应由 `vole-runtime` 决定。
//! - `CaughtError::Error` 不是 JS exception，只能降级为字符串值。
use rquickjs::{
    CatchResultExt, CaughtError, Ctx, Error, Exception, Function, IntoJs, Result, Value,
    function::Constructor,
};

/// Web API 常用 DOMException 名称。
///
/// 这里不定义完整 DOMException class；真正的全局 `DOMException` 仍由
/// `vole-global-dom-exception` 注册。本模块只提供“如果当前 context 已注册
/// DOMException，则按 Web API 方式构造；否则退化为普通 Error 对象”的底层
/// helper，避免 `core` 反向依赖具体 global binding。
pub mod dom_exception_name {
    pub const ABORT: &str = "AbortError";
    pub const DATA_CLONE: &str = "DataCloneError";
    pub const DATA: &str = "DataError";
    pub const ENCODING: &str = "EncodingError";
    pub const INVALID_ACCESS: &str = "InvalidAccessError";
    pub const INVALID_CHARACTER: &str = "InvalidCharacterError";
    pub const NOT_SUPPORTED: &str = "NotSupportedError";
    pub const OPERATION: &str = "OperationError";
    pub const QUOTA_EXCEEDED: &str = "QuotaExceededError";
    pub const SECURITY: &str = "SecurityError";
    pub const SYNTAX: &str = "SyntaxError";
    pub const TIMEOUT: &str = "TimeoutError";
}

/// 将 rquickjs 错误转换成当前 JS context 中可保存/传递的 JS 值。
pub trait ErrorExtensions<'js> {
    fn into_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>>;
}

impl<'js> ErrorExtensions<'js> for Error {
    fn into_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Err::<(), _>(self).catch(ctx).unwrap_err().into_value(ctx)
    }
}

impl<'js> ErrorExtensions<'js> for CaughtError<'js> {
    fn into_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(match self {
            CaughtError::Error(err) => err.to_string().into_js(ctx)?,
            CaughtError::Exception(ex) => ex.into_value(),
            CaughtError::Value(val) => val,
        })
    }
}

/// 构造一个 JS `DOMException` 值。
///
/// 如果当前 context 中已经存在 `globalThis.DOMException`，优先调用该构造器，
/// 以便获得正确的 prototype、legacy `code` 和 `name/message` 行为。
/// 如果尚未注册 DOMException，则退化为普通 `Error` 对象，并补上 `name`
/// 字段。这个退化路径用于 core helper 被早期初始化阶段调用的场景。
pub fn dom_exception_value<'js>(ctx: &Ctx<'js>, name: &str, message: &str) -> Result<Value<'js>> {
    if let Ok(ctor) = ctx.globals().get::<_, Constructor>("DOMException") {
        return ctor.construct((message, name));
    }

    let object = Exception::from_message(ctx.clone(), message)?.into_object();
    object.set("name", name)?;
    Ok(object.into_value())
}

/// 抛出一个 DOMException。
///
/// 该函数返回泛型 `Result<T>`，方便在 API 边界直接写：
///
/// ```ignore
/// return vole_core::error::throw_dom_exception(
///     &ctx,
///     dom_exception_name::INVALID_CHARACTER,
///     "The string to be encoded contains invalid characters",
/// );
/// ```
pub fn throw_dom_exception<'js, T>(ctx: &Ctx<'js>, name: &str, message: &str) -> Result<T> {
    match dom_exception_value(ctx, name, message) {
        Ok(value) => Err(ctx.throw(value)),
        Err(_) => Err(Exception::throw_message(ctx, message)),
    }
}

/// 抛出普通 `Error`。
pub fn throw_error<'js, T>(ctx: &Ctx<'js>, message: &str) -> Result<T> {
    Err(Exception::throw_message(ctx, message))
}

/// 抛出 `TypeError`。
pub fn throw_type_error<'js, T>(ctx: &Ctx<'js>, message: &str) -> Result<T> {
    Err(Exception::throw_type(ctx, message))
}

/// 抛出 `RangeError`。
pub fn throw_range_error<'js, T>(ctx: &Ctx<'js>, message: &str) -> Result<T> {
    Err(Exception::throw_range(ctx, message))
}

/// 用任意 JS 值 reject Promise。
pub fn reject_value<'js>(reject: &Function<'js>, value: Value<'js>) -> Result<()> {
    reject.call::<_, ()>((value,))
}

/// 把 `rquickjs::Error` 转成 JS 值后 reject Promise。
pub fn reject_error<'js>(ctx: &Ctx<'js>, reject: &Function<'js>, error: Error) -> Result<()> {
    let value = error.into_value(ctx)?;
    reject_value(reject, value)
}

/// 构造 DOMException 并 reject Promise。
pub fn reject_dom_exception<'js>(
    ctx: &Ctx<'js>,
    reject: &Function<'js>,
    name: &str,
    message: &str,
) -> Result<()> {
    let value = dom_exception_value(ctx, name, message)?;
    reject_value(reject, value)
}
