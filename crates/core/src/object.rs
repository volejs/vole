// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! JS object 访问和转换辅助工具。
//!
//! rquickjs 的 `Object::get` 对 `undefined`、缺失属性、类型转换失败等情况
//! 的语义比较底层。Web API binding 通常需要更明确的字典解析行为，因此
//! 本模块提供统一的 object helper。
//!
//! 设计目标：
//! - 统一 optional / required 属性读取。
//! - 统一“不是 object”时抛 `TypeError` 的错误消息。
//! - 提供 Map/entries 互转辅助，服务 Headers、URLSearchParams、FormData 等
//!   具有 entries 语义的 API。
//!
//! 注意：这里的 `get_optional` 使用 rquickjs 的 `Option<V>` 语义；如果需要
//! 严格区分 `undefined` 和 `null`，应使用 [`crate::option`] 或 API 自己的
//! WebIDL dictionary 解析。
use std::collections::BTreeMap;

use rquickjs::{
    atom::PredefinedAtom, function::IntoJsFunc, prelude::Func, Array, Coerced, Ctx, Error,
    Exception, FromJs, IntoAtom, IntoJs, Object, Result, Undefined, Value,
};

use crate::primordials::{BasePrimordials, Primordial};

/// JS object/value 的通用读取扩展。
pub trait ObjectExt<'js> {
    /// 读取可选属性。
    ///
    /// 这个方法适合“缺失属性即 None”的场景，不适合需要区分 `null` 和
    /// `undefined` 的严格 WebIDL 场景。
    fn get_optional<K: IntoAtom<'js> + Clone, V: FromJs<'js>>(&self, k: K) -> Result<Option<V>>;
    /// 读取必填属性，缺失时抛 `TypeError`。
    fn get_required<K: AsRef<str>, V: FromJs<'js>>(
        &self,
        k: K,
        object_name: &'static str,
    ) -> Result<V>;
    /// 把当前值转换为 object，失败时抛带 API 名称的 `TypeError`。
    fn into_object_or_throw(self, ctx: &Ctx<'js>, object_name: &'static str)
        -> Result<Object<'js>>;
}

impl<'js> ObjectExt<'js> for Object<'js> {
    fn get_optional<K: IntoAtom<'js> + Clone, V: FromJs<'js> + Sized>(
        &self,
        k: K,
    ) -> Result<Option<V>> {
        self.get::<K, Option<V>>(k)
    }

    fn get_required<K: AsRef<str>, V: FromJs<'js>>(
        &self,
        k: K,
        object_name: &'static str,
    ) -> Result<V> {
        let k = k.as_ref();
        self.get::<&str, Option<V>>(k)?.ok_or_else(|| {
            Exception::throw_type(
                self.ctx(),
                &[object_name, " '", k, "' property required"].concat(),
            )
        })
    }

    fn into_object_or_throw(self, _: &Ctx<'js>, _: &'static str) -> Result<Object<'js>> {
        Ok(self)
    }
}

impl<'js> ObjectExt<'js> for Value<'js> {
    fn get_optional<K: IntoAtom<'js> + Clone, V: FromJs<'js>>(&self, k: K) -> Result<Option<V>> {
        if let Some(obj) = self.as_object() {
            return obj.get_optional(k);
        }
        Ok(None)
    }

    fn get_required<K: AsRef<str>, V: FromJs<'js>>(
        &self,
        k: K,
        object_name: &'static str,
    ) -> Result<V> {
        self.as_object()
            .ok_or_else(|| not_a_object_error(self.ctx(), object_name))?
            .get_required(k, object_name)
    }

    fn into_object_or_throw(
        self,
        ctx: &Ctx<'js>,
        object_name: &'static str,
    ) -> Result<Object<'js>> {
        self.into_object()
            .ok_or_else(|| not_a_object_error(ctx, object_name))
    }
}

/// 构造“不是 object”的标准 `TypeError`。
pub fn not_a_object_error(ctx: &Ctx<'_>, object_name: &str) -> Error {
    Exception::throw_type(ctx, &[object_name, " is not an object"].concat())
}

/// JS `Proxy` 构造辅助。
///
/// 该类型只封装 target/options 的创建和 getter/setter 注入，不处理具体代理
/// 行为。调用者应确保代理不会破坏 Web API 的规范 invariants。
pub struct Proxy<'js> {
    target: Value<'js>,
    options: Object<'js>,
}

impl<'js> IntoJs<'js> for Proxy<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        BasePrimordials::get(ctx)?
            .constructor_proxy
            .construct::<_, Value>((self.target, self.options))
    }
}

impl<'js> Proxy<'js> {
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        let options = Object::new(ctx.clone())?;
        Ok(Self {
            target: Undefined.into_value(ctx),
            options,
        })
    }

    pub fn with_target(ctx: Ctx<'js>, target: Value<'js>) -> Result<Self> {
        let options = Object::new(ctx)?;
        Ok(Self { target, options })
    }

    pub fn setter<T, P: 'js>(&self, setter: Func<T, P>) -> Result<()>
    where
        T: IntoJsFunc<'js, P> + 'js,
    {
        self.options.set(PredefinedAtom::Setter, setter)?;
        Ok(())
    }

    pub fn getter<T, P: 'js>(&self, getter: Func<T, P>) -> Result<()>
    where
        T: IntoJsFunc<'js, P> + 'js,
    {
        self.options.set(PredefinedAtom::Getter, getter)?;
        Ok(())
    }
}

/// 将 Rust iterator 转成 JS entries array。
pub fn map_to_entries<'js, K, V, M>(ctx: &Ctx<'js>, map: M) -> Result<Array<'js>>
where
    M: IntoIterator<Item = (K, V)>,
    K: IntoJs<'js>,
    V: IntoJs<'js>,
{
    let array = Array::new(ctx.clone())?;
    for (idx, (key, value)) in map.into_iter().enumerate() {
        let entry = Array::new(ctx.clone())?;
        entry.set(0, key)?;
        entry.set(1, value)?;
        array.set(idx, entry)?;
    }

    Ok(array)
}

/// 将 JS entries array 转成 `BTreeMap<String, Coerced<String>>`。
pub fn array_to_btree_map<'js>(
    ctx: &Ctx<'js>,
    array: Array<'js>,
) -> Result<BTreeMap<String, Coerced<String>>> {
    let value = object_from_entries(ctx, array)?;
    let value = value.into_value();
    BTreeMap::from_js(ctx, value)
}

/// 根据 JS entries array 构造普通 object。
pub fn object_from_entries<'js>(ctx: &Ctx<'js>, array: Array<'js>) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    for value in array.into_iter().flatten() {
        if let Some(entry) = value.as_array() {
            if let Ok(key) = entry.get::<Value>(0) {
                if let Ok(value) = entry.get::<Value>(1) {
                    let _ = obj.set(key, value); //ignore result of failed
                }
            }
        }
    }
    Ok(obj)
}
