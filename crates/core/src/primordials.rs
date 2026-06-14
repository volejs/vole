//! JS primordial 缓存。
//!
//! “Primordial” 指 JS runtime 初始化时就存在的内建构造器、原型和函数，
//! 例如 `Object`、`TypeError`、`ArrayBuffer.isView`、`Reflect.ownKeys`。
//!
//! JS 用户代码可以修改这些全局对象：
//!
//! ```text
//! ArrayBuffer.isView = () => false
//! Promise.prototype.then = () => {}
//! globalThis.TypeError = null
//! ```
//!
//! 如果 native binding 每次都从 `globalThis` 动态读取这些对象，就会受到用户
//! 代码污染。`primordials` 的设计是在初始化阶段把原始引用缓存到
//! `Ctx::userdata` 中，后续内部实现统一从这里读取。
//!
//! 使用约束：
//! - 使用任何 primordial 前必须先调用 `BasePrimordials::init(ctx)`。
//! - 缓存值只在当前 QuickJS context 的 `'js` 生命周期内有效。
//! - 这里保存的是“原始引用”，不是安全沙箱；如果初始化发生在用户污染之后，
//!   仍然会缓存被污染后的对象。因此 runtime 应尽早初始化它。

use std::any::type_name;

use rquickjs::{
    atom::PredefinedAtom, function::Constructor, runtime::UserDataGuard, Ctx, Exception, Function,
    JsLifetime, Object, Result,
};

use crate::result::ResultExt;

#[derive(JsLifetime)]
/// 基础 JS primordial 集合。
///
/// 该结构只收集多个 binding 都会用到的内建对象。某个 API 专用的 primordial
/// 应定义在对应模块中，例如 Streams 的 `PromisePrimordials`。
pub struct BasePrimordials<'js> {
    // Constructors
    pub constructor_map: Constructor<'js>,
    pub constructor_set: Constructor<'js>,
    pub constructor_date: Constructor<'js>,
    pub constructor_error: Constructor<'js>,
    pub constructor_type_error: Constructor<'js>,
    pub constructor_range_error: Constructor<'js>,
    pub constructor_regexp: Constructor<'js>,
    pub constructor_uint8array: Constructor<'js>,
    pub constructor_array_buffer: Constructor<'js>,
    pub constructor_proxy: Constructor<'js>,
    pub constructor_object: Constructor<'js>,
    pub constructor_bool: Constructor<'js>,
    pub constructor_number: Constructor<'js>,
    pub constructor_string: Constructor<'js>,

    // Prototypes
    pub prototype_object: Object<'js>,
    pub prototype_date: Object<'js>,
    pub prototype_regexp: Object<'js>,
    pub prototype_set: Object<'js>,
    pub prototype_map: Object<'js>,
    pub prototype_error: Object<'js>,

    // Functions
    pub function_array_from: Function<'js>,
    pub function_array_buffer_is_view: Function<'js>,
    pub function_get_own_property_descriptor: Function<'js>,
    pub function_reflect_own_keys: Function<'js>,
    pub function_parse_int: Function<'js>,
    pub function_parse_float: Function<'js>,
    pub prototype_iterator: Object<'js>,
}

pub trait Primordial<'js>
where
    Self: Sized + JsLifetime<'js>,
{
    /// 从当前 context 的 userdata 中获取已初始化的 primordial。
    ///
    /// 如果未初始化，会返回带类型名的错误，提示调用方先执行 `init`。
    fn get<'a>(ctx: &'a Ctx<'js>) -> Result<UserDataGuard<'a, Self>> {
        let userdata = ctx.userdata::<Self>().or_throw_msg(
            ctx,
            &[
                "Userdata of ",
                type_name::<Self>(),
                " not initialized. Call init(&ctx) on this type.",
            ]
            .concat(),
        )?;

        Ok(userdata)
    }

    /// 初始化 primordial 并保存到 `Ctx::userdata`。
    ///
    /// 同一 context 内只会初始化一次；重复调用是 no-op。
    fn init<'a>(ctx: &'a Ctx<'js>) -> Result<()> {
        if ctx.userdata::<Self>().is_none() {
            let primoridals = Self::new(ctx)?;
            let _ = ctx.store_userdata(primoridals);
        }

        Ok(())
    }
    /// 从当前 JS global 中收集 primordial。
    ///
    /// 实现者应只在这里读取 global/prototype/function，并尽量避免执行用户代码。
    fn new(ctx: &Ctx<'js>) -> Result<Self>;
}

impl<'js> Primordial<'js> for BasePrimordials<'js> {
    fn new(ctx: &Ctx<'js>) -> Result<Self> {
        let globals = ctx.globals();

        let constructor_object: Constructor = globals.get(PredefinedAtom::Object)?;
        let prototype_object: Object = constructor_object.get(PredefinedAtom::Prototype)?;

        let constructor_proxy: Constructor = globals.get(PredefinedAtom::Proxy)?;

        let function_get_own_property_descriptor: Function =
            constructor_object.get(PredefinedAtom::GetOwnPropertyDescriptor)?;

        let constructor_date: Constructor = globals.get(PredefinedAtom::Date)?;
        let prototype_date: Object = constructor_date.get(PredefinedAtom::Prototype)?;

        let constructor_map: Constructor = globals.get(PredefinedAtom::Map)?;
        let prototype_map: Object = constructor_map.get(PredefinedAtom::Prototype)?;

        let constructor_set: Constructor = globals.get(PredefinedAtom::Set)?;
        let prototype_set: Object = constructor_set.get(PredefinedAtom::Prototype)?;

        let constructor_regexp: Constructor = globals.get(PredefinedAtom::RegExp)?;
        let prototype_regexp: Object = constructor_regexp.get(PredefinedAtom::Prototype)?;

        let constructor_uint8array: Constructor = globals.get(PredefinedAtom::Uint8Array)?;
        let constructor_arraybuffer: Constructor = globals.get(PredefinedAtom::ArrayBuffer)?;

        let constructor_error: Constructor = globals.get(PredefinedAtom::Error)?;
        let constructor_type_error: Constructor = ctx.globals().get(PredefinedAtom::TypeError)?;
        let constructor_range_error: Constructor = ctx.globals().get(PredefinedAtom::RangeError)?;
        let prototype_error: Object = constructor_error.get(PredefinedAtom::Prototype)?;

        let constructor_array: Object = globals.get(PredefinedAtom::Array)?;
        let function_array_from: Function = constructor_array.get(PredefinedAtom::From)?;

        let constructor_array_buffer: Object = globals.get(PredefinedAtom::ArrayBuffer)?;
        let function_array_buffer_is_view: Function = constructor_array_buffer.get("isView")?;

        let constructor_bool: Constructor = globals.get(PredefinedAtom::Boolean)?;

        let constructor_number: Constructor = globals.get(PredefinedAtom::Number)?;
        let function_parse_float: Function = constructor_number.get("parseFloat")?;
        let function_parse_int: Function = constructor_number.get("parseInt")?;

        let constructor_string: Constructor = globals.get(PredefinedAtom::String)?;

        let reflect: Object = globals.get("Reflect")?;
        let function_reflect_own_keys: Function = reflect.get("ownKeys")?;

        // Walk to %IteratorPrototype% via an array iterator.
        let array = rquickjs::Array::new(ctx.clone())?;
        let iter_fn: Function = array
            .as_object()
            .get(rquickjs::atom::PredefinedAtom::SymbolIterator)?;
        let array_iter: Object = iter_fn.call((rquickjs::function::This(array),))?;
        let prototype_iterator = array_iter
            .get_prototype()
            .and_then(|p| p.get_prototype())
            .ok_or_else(|| Exception::throw_internal(ctx, "missing %IteratorPrototype%"))?;

        Ok(Self {
            constructor_map,
            constructor_set,
            constructor_date,
            constructor_proxy,
            constructor_error,
            constructor_type_error,
            constructor_range_error,
            constructor_regexp,
            constructor_uint8array,
            constructor_array_buffer: constructor_arraybuffer,
            constructor_object,
            constructor_bool,
            constructor_number,
            constructor_string,
            prototype_object,
            prototype_date,
            prototype_regexp,
            prototype_set,
            prototype_map,
            prototype_error,
            function_array_from,
            function_array_buffer_is_view,
            function_get_own_property_descriptor,
            function_reflect_own_keys,
            function_parse_float,
            function_parse_int,
            prototype_iterator,
        })
    }
}
