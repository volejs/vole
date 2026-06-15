//! WebIDL 参数中 `undefined`、`null` 和缺失值的辅助类型。
//!
//! Rust 的 `Option<T>` 在 rquickjs 中通常会把“缺失参数”和某些 JS 空值场景
//! 合并处理，但 WebIDL 很多地方需要严格区分：
//! - 参数缺失
//! - 显式传入 `undefined`
//! - 显式传入 `null`
//!
//! 本模块提供几个 wrapper，用于在 `FromJs` / `IntoJs` / 函数参数解析时保留
//! 这些差异。
//!
//! 使用建议：
//! - 字典属性中“只有 undefined 表示不存在”时，用 [`Undefined`]。
//! - 返回值需要把 Rust `None` 表达成 JS `null` 时，用 [`Null`]。
//! - 函数参数允许缺失或 `null`，但不把 `undefined` 当缺失时，用
//!   [`NullableOpt`]。

use rquickjs::{
    Ctx, FromJs, IntoJs, JsLifetime, Result, Type, Value,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

/// 将 JS `undefined` 视为 `None`，但不会把 `null` 视为 `None`。
///
/// 这正好匹配很多 WebIDL dictionary 的属性读取语义：属性不存在或为
/// `undefined` 时使用默认值，但 `null` 需要继续参与类型转换。
#[derive(Clone)]
pub struct Undefined<T>(pub Option<T>);

impl<'js, T: FromJs<'js>> FromJs<'js> for Undefined<T> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.type_of() == Type::Undefined {
            Ok(Self(None))
        } else {
            Ok(Self(Some(FromJs::from_js(ctx, value)?)))
        }
    }
}

impl<'js, T: IntoJs<'js>> IntoJs<'js> for Undefined<T> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self.0 {
            None => Ok(Value::new_undefined(ctx.clone())),
            Some(val) => val.into_js(ctx),
        }
    }
}

impl<T> Default for Undefined<T> {
    fn default() -> Self {
        Self(None)
    }
}

unsafe impl<'js, T: JsLifetime<'js>> JsLifetime<'js> for Undefined<T> {
    type Changed<'to> = Undefined<T::Changed<'to>>;
}

impl<'js, T: Trace<'js>> Trace<'js> for Undefined<T> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

/// 把 Rust `None` 序列化成 JS `null`。
///
/// 适合 Web API getter 需要返回 nullable IDL 类型的场景。
#[derive(Clone)]
pub struct Null<T>(pub Option<T>);

impl<'js, T: FromJs<'js>> FromJs<'js> for Null<T> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.type_of() == Type::Null {
            Ok(Self(None))
        } else {
            Ok(Self(Some(FromJs::from_js(ctx, value)?)))
        }
    }
}

impl<'js, T: IntoJs<'js>> IntoJs<'js> for Null<T> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self.0 {
            None => Ok(Value::new_null(ctx.clone())),
            Some(val) => val.into_js(ctx),
        }
    }
}

unsafe impl<'js, T: JsLifetime<'js>> JsLifetime<'js> for Null<T> {
    type Changed<'to> = Null<T::Changed<'to>>;
}

impl<'js, T: Trace<'js>> Trace<'js> for Null<T> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.0.trace(tracer)
    }
}

/// 用于函数参数解析：缺失参数或 `null` 表示 `None`，但显式传入
/// `undefined` 会继续按 `T` 转换。
pub struct NullableOpt<T>(pub Option<T>);

impl<'js, T: FromJs<'js>> FromParam<'js> for NullableOpt<T> {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::optional()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        if !params.is_empty() {
            let arg = params.arg();
            if arg.is_null() {
                Ok(NullableOpt(None))
            } else {
                let ctx = params.ctx().clone();
                Ok(NullableOpt(Some(T::from_js(&ctx, arg)?)))
            }
        } else {
            Ok(NullableOpt(None))
        }
    }
}
