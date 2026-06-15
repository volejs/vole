// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! rquickjs class / WebIDL class 辅助工具。
//!
//! 这个模块封装 class 定义、class name 获取、自定义 inspect symbol、
//! WebIDL iterator 原型设置等重复逻辑。它不应该包含具体 Web API 的业务
//! 状态机，只提供 class 层面的结构性辅助。
//!
//! 使用注意：
//! - 这里的 helper 通常假设 [`BasePrimordials`](crate::primordials::BasePrimordials)
//!   已经初始化。
//! - WebIDL iterator 的 prototype 链比较敏感，应统一使用
//!   [`WebIdlIteratorExtension`]，避免每个 binding 手动拼接。
//! - `CUSTOM_INSPECT_SYMBOL_DESCRIPTION` 目前沿用 LLRT 兼容语义，后续如果
//!   vole 定义自己的 inspect 协议，可以在这里统一调整。
use rquickjs::{
    Array, Class, Ctx, Function, Object, Result, Symbol, Value, atom::PredefinedAtom,
    class::JsClass, object::Accessor, object::Property, prelude::This,
};

use crate::{
    object::ObjectExt,
    primordials::{BasePrimordials, Primordial},
    result::OptionExt,
};

pub static CUSTOM_INSPECT_SYMBOL_DESCRIPTION: &str = "llrt.inspect.custom";

/// 为 Rust class 提供 JS iterator 的统一接口。
///
/// 实现者只需要返回 entries array，本 trait 会通过 JS 的 `Array.prototype.values`
/// 得到 iterator 对象。
pub trait IteratorDef<'js>
where
    Self: 'js + JsClass<'js> + Sized,
{
    fn js_entries(&self, ctx: Ctx<'js>) -> Result<Array<'js>>;

    fn js_iterator(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let value = self.js_entries(ctx)?;
        let obj = value.as_object();
        let values_fn: Function = obj.get(PredefinedAtom::Values)?;
        values_fn.call((This(value),))
    }
}

/// 尝试读取 JS 值的 constructor name。
///
/// 这用于 console formatting、错误消息或调试输出。不要把它用于安全判断，
/// 因为 JS 用户可以修改 `constructor` 或 `name`。
pub fn get_class_name(value: &Value) -> Result<Option<String>> {
    value
        .get_optional::<_, Object>(PredefinedAtom::Constructor)?
        .and_then_ok(|ctor| ctor.get_optional::<_, String>(PredefinedAtom::Name))
}

#[inline(always)]
/// 如果 JS 值是指定 rquickjs class 的实例，则返回该 class。
///
/// 这比直接 `Class::from_value` 更适合宽松解析：非目标实例返回 `Ok(None)`，
/// 只有底层转换失败才返回错误。
pub fn get_class<'js, C>(provided: &Value<'js>) -> Result<Option<Class<'js, C>>>
where
    C: JsClass<'js>,
{
    if provided
        .as_object()
        .map(|p| p.instance_of::<C>())
        .unwrap_or_default()
    {
        return Ok(Some(Class::<C>::from_value(provided)?));
    }
    Ok(None)
}

/// 为 class 定义自定义 inspect 的扩展点。
///
/// 这个能力主要服务 console / util.format 一类调试输出，不应影响对象的
/// 标准 Web API 行为。
pub trait CustomInspectExtension<'js> {
    fn define_with_custom_inspect(globals: &Object<'js>) -> Result<()>;
}

/// class 自定义 inspect 输出的实现接口。
pub trait CustomInspect<'js>
where
    Self: JsClass<'js>,
{
    fn custom_inspect(&self, ctx: Ctx<'js>) -> Result<Object<'js>>;
}

impl<'js, C> CustomInspectExtension<'js> for Class<'js, C>
where
    C: JsClass<'js> + CustomInspect<'js> + 'js,
{
    fn define_with_custom_inspect(globals: &Object<'js>) -> Result<()> {
        Self::define(globals)?;
        let custom_inspect_symbol =
            Symbol::new_global(globals.ctx().clone(), CUSTOM_INSPECT_SYMBOL_DESCRIPTION)?;
        if let Some(proto) = Class::<C>::prototype(globals.ctx())? {
            proto.prop(
                custom_inspect_symbol,
                Accessor::from(|this: This<Class<'js, C>>, ctx| this.borrow().custom_inspect(ctx)),
            )?;
        }
        Ok(())
    }
}

/// 将 class 注册成 WebIDL pair iterator。
///
/// 这个扩展会完成几件事：
/// - 先把 class 注册到 globals。
/// - 再移除 constructor，因为 WebIDL iterator 通常不直接暴露构造器。
/// - 把 prototype 接到 `%IteratorPrototype%`，使其表现为标准 iterator。
/// - 按 WebIDL 要求把 `next` 重新定义成 enumerable。
///
/// 使用前提：目标 class 必须通过 `#[rquickjs::methods]` 定义
/// `next(&mut self, ctx) -> Result<Object>` 方法。
pub trait WebIdlIteratorExtension<'js> {
    fn define_as_webidl_iterator(globals: &Object<'js>, name: &str) -> Result<()>;
}

impl<'js, C> WebIdlIteratorExtension<'js> for Class<'js, C>
where
    C: JsClass<'js> + 'js,
{
    fn define_as_webidl_iterator(globals: &Object<'js>, name: &str) -> Result<()> {
        let ctx = globals.ctx();
        Self::define(globals)?;
        // Iterator class is not exposed to JS.
        globals.remove(name)?;
        if let Some(proto) = Class::<C>::prototype(ctx)? {
            let iterator_proto = BasePrimordials::get(ctx)?.prototype_iterator.clone();
            proto.set_prototype(Some(&iterator_proto))?;
            let next_fn: Function = proto.get("next")?;
            proto.prop(
                "next",
                Property::from(next_fn)
                    .writable()
                    .enumerable()
                    .configurable(),
            )?;
        }
        Ok(())
    }
}
