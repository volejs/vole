//! Promise 操作辅助工具。
//!
//! Web API 规范大量使用“返回 promise”、“promise fulfilled 后继续步骤”、
//! “同步异常转成 rejected promise”等算法。直接在每个 binding 中手写这些
//! 逻辑容易出现不一致，也容易被用户污染的 `Promise.prototype.then` 影响。
//!
//! 本模块提供：
//! - [`PromisePrimordials`]：缓存 Promise 构造器和原始 `then/resolve/reject/all`。
//! - [`ResolveablePromise`]：Rust 侧持有 promise 及其 resolve/reject。
//! - `upon_promise` 系列：用缓存的 `Promise.prototype.then` 继续异步步骤。
//!
//! 注意事项：
//! - 使用前必须初始化 [`PromisePrimordials`]。
//! - 这里只做 promise 状态桥接，不负责 `unhandledrejection` 分发。
//! - 如果 closure 捕获 JS 值，必须确保相关类型实现 `Trace` 或受 rquickjs
//!   生命周期保护。

use std::{cell::Cell, rc::Rc};

use crate::primordials::Primordial;
use rquickjs::{
    Ctx, Error, FromJs, Function, IntoJs, JsLifetime, Object, Promise, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::Constructor,
    prelude::{IntoArg, OnceFn, This},
    promise::PromiseState,
};

/// 用指定 JS 值创建 rejected promise。
pub fn promise_rejected_with<'js>(
    primordials: &PromisePrimordials<'js>,
    value: Value<'js>,
) -> Result<Promise<'js>> {
    primordials
        .promise_reject
        .call((This(primordials.promise_constructor.clone()), value))
}

/// 使用当前 pending exception 创建 rejected promise。
pub fn promise_rejected_catch<'js>(
    ctx: &Ctx<'js>,
    promise_primordials: &PromisePrimordials<'js>,
) -> Result<Promise<'js>> {
    promise_rejected_with(promise_primordials, ctx.catch())
}

/// 用指定 JS Error constructor 创建 rejected promise。
pub fn promise_rejected_with_constructor<'js, T: From<Error>>(
    constructor: &Constructor<'js>,
    promise_primordials: &PromisePrimordials<'js>,
    msg: &str,
) -> std::result::Result<Promise<'js>, T> {
    let e: Value = constructor.call((msg,))?;
    Ok(promise_rejected_with(promise_primordials, e)?)
}

/// 将 `Result<Value>` 转成 resolved/rejected promise。
///
/// 如果 `value` 是 `Error::Exception`，会捕获当前 JS exception 并转成
/// rejected promise；其他 Rust 错误会原样返回。
pub fn promise_resolved_with<'js>(
    ctx: &Ctx<'js>,
    primordials: &PromisePrimordials<'js>,
    value: Result<Value<'js>>,
) -> Result<Promise<'js>> {
    match value {
        Ok(value) => primordials
            .promise_resolve
            .call((This(primordials.promise_constructor.clone()), value)),
        Err(Error::Exception) => primordials
            .promise_reject
            .call((This(primordials.promise_constructor.clone()), ctx.catch())),
        Err(err) => Err(err),
    }
}

#[derive(JsLifetime, Clone)]
/// Promise 相关 primordial。
///
/// 这些引用在初始化时从 JS global 缓存，避免后续用户改写 `Promise` 或
/// `Promise.prototype.then` 影响内部算法。
pub struct PromisePrimordials<'js> {
    pub promise_constructor: Constructor<'js>,
    pub promise_resolve: Function<'js>,
    pub promise_reject: Function<'js>,
    pub promise_all: Function<'js>,
    pub promise_resolved_with_undefined: Promise<'js>,
    pub promise_prototype_then: Function<'js>,
}

impl<'js> Trace<'js> for PromisePrimordials<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.promise_constructor.trace(tracer);
        self.promise_resolve.trace(tracer);
        self.promise_reject.trace(tracer);
        self.promise_all.trace(tracer);
        self.promise_resolved_with_undefined.trace(tracer);
        self.promise_prototype_then.trace(tracer);
    }
}

impl<'js> Primordial<'js> for PromisePrimordials<'js> {
    fn new(ctx: &Ctx<'js>) -> Result<Self>
    where
        Self: Sized,
    {
        let promise_constructor: Constructor<'js> = ctx.globals().get(PredefinedAtom::Promise)?;
        let promise_resolve: Function<'js> = promise_constructor.get("resolve")?;
        let promise_reject: Function<'js> = promise_constructor.get("reject")?;
        let promise_all: Function<'js> = promise_constructor.get("all")?;
        let promise_prototype_then: Function<'js> = promise_constructor
            .get::<_, Object>("prototype")?
            .get("then")?;

        let promise_resolved_with_undefined = promise_resolve.call((
            This(promise_constructor.clone()),
            Value::new_undefined(ctx.clone()),
        ))?;

        Ok(Self {
            promise_constructor,
            promise_resolve,
            promise_reject,
            promise_all,
            promise_resolved_with_undefined,
            promise_prototype_then,
        })
    }
}

// https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled
/// 在 promise settled 后执行一次回调。
///
/// 回调收到 `Ok(value)` 表示 fulfilled，收到 `Err(reason)` 表示 rejected。
/// 该函数使用缓存的 `Promise.prototype.then`，而不是从 promise 对象上动态
/// 读取 `then`。
pub fn upon_promise<'js, Input: FromJs<'js> + 'js, Output: IntoJs<'js> + 'js>(
    ctx: Ctx<'js>,
    promise: Promise<'js>,
    then: impl FnOnce(Ctx<'js>, std::result::Result<Input, Value<'js>>) -> Result<Output> + 'js,
) -> Result<Promise<'js>> {
    let promise_then = PromisePrimordials::get(&ctx)?
        .promise_prototype_then
        .clone();
    let then_cb = Rc::new(Cell::new(Some(then)));
    let then_cb2 = then_cb.clone();
    promise_then.call((
        This(promise),
        Function::new(
            ctx.clone(),
            OnceFn::new(move |ctx, input| {
                then_cb
                    .take()
                    .expect("Promise.then should only call either resolve or reject")(
                    ctx,
                    Ok(input),
                )
            }),
        ),
        Function::new(
            ctx,
            OnceFn::new(move |ctx, e: Value<'js>| {
                then_cb2
                    .take()
                    .expect("Promise.then should only call either resolve or reject")(
                    ctx, Err(e)
                )
            }),
        ),
    ))
}

/// 在 promise fulfilled 后执行一次回调。
///
/// rejected 情况不会调用该回调，而是按 JS promise 链规则继续传播。
pub fn upon_promise_fulfilment<'js, Input: FromJs<'js> + 'js, Output: IntoJs<'js> + 'js>(
    ctx: Ctx<'js>,
    promise: Promise<'js>,
    then: impl FnOnce(Ctx<'js>, Input) -> Result<Output> + 'js,
) -> Result<Promise<'js>> {
    let promise_then = PromisePrimordials::get(&ctx)?
        .promise_prototype_then
        .clone();
    promise_then.call((This(promise), Function::new(ctx.clone(), OnceFn::new(then))))
}

#[derive(Debug, JsLifetime, Clone)]
/// Rust 侧可解析/拒绝的 Promise 句柄。
///
/// 该类型持有 promise 本身以及可选的 resolve/reject 函数。对于已经 resolved
/// 或 rejected 的静态 promise，resolve/reject 可以为空。
pub struct ResolveablePromise<'js> {
    pub promise: Promise<'js>,
    resolve: Option<Function<'js>>,
    reject: Option<Function<'js>>,
}

impl<'js> ResolveablePromise<'js> {
    /// 创建一个新的 pending promise。
    pub fn new(ctx: &Ctx<'js>) -> Result<Self> {
        let (promise, resolve, reject) = Promise::new(ctx)?;
        Ok(Self {
            promise,
            resolve: Some(resolve),
            reject: Some(reject),
        })
    }

    /// 构造一个已 fulfilled(undefined) 的 promise wrapper。
    pub fn resolved_with_undefined(primordials: &PromisePrimordials<'js>) -> Self {
        Self {
            promise: primordials.promise_resolved_with_undefined.clone(),
            resolve: None,
            reject: None,
        }
    }

    /// 构造一个已 rejected 的 promise wrapper。
    pub fn rejected_with(primordials: &PromisePrimordials<'js>, error: Value<'js>) -> Result<Self> {
        Ok(Self {
            promise: promise_rejected_with(primordials, error)?,
            resolve: None,
            reject: None,
        })
    }

    /// 使用 JS Error constructor 构造一个已 rejected 的 promise wrapper。
    pub fn rejected_with_constructor(
        primordials: &PromisePrimordials<'js>,
        constructor: &Constructor<'js>,
        msg: &str,
    ) -> Result<Self> {
        Ok(Self {
            promise: promise_rejected_with_constructor::<rquickjs::Error>(
                constructor,
                primordials,
                msg,
            )?,
            resolve: None,
            reject: None,
        })
    }

    /// resolve 当前 promise。
    ///
    /// 如果该 wrapper 没有 resolve 函数，调用是 no-op。
    pub fn resolve(&self, value: impl IntoArg<'js>) -> Result<()> {
        if let Some(resolve) = &self.resolve {
            let () = resolve.call((value,))?;
        }
        Ok(())
    }

    /// resolve 当前 promise 为 `undefined`。
    pub fn resolve_undefined(&self) -> Result<()> {
        if let Some(resolve) = &self.resolve {
            let () = resolve.call((rquickjs::Undefined,))?;
        }
        Ok(())
    }

    /// reject 当前 promise。
    ///
    /// 如果该 wrapper 没有 reject 函数，调用是 no-op。
    pub fn reject(&self, value: impl IntoArg<'js>) -> Result<()> {
        if let Some(reject) = &self.reject {
            let () = reject.call((value,))?;
        }
        Ok(())
    }

    /// 用指定 JS Error constructor 创建错误并 reject 当前 promise。
    pub fn reject_with_constructor(&self, constructor: &Constructor<'js>, msg: &str) -> Result<()> {
        if let Some(reject) = &self.reject {
            let e: Value = constructor.call((msg,))?;
            let () = reject.call((e,))?;
        }
        Ok(())
    }

    /// 判断 promise 是否仍处于 pending 状态。
    pub fn is_pending(&self) -> bool {
        self.promise.state() == PromiseState::Pending
    }

    /// 标记 promise rejection 已被处理。
    ///
    /// 这会挂一个空的 catch handler，常用于内部 promise，避免误触发
    /// unhandled rejection。
    pub fn set_is_handled(&self) -> Result<()> {
        self.promise.catch()?.call((
            This(self.promise.clone()),
            Function::new(self.promise.ctx().clone(), || {}),
        ))
    }
}

impl<'js> Trace<'js> for ResolveablePromise<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.promise.trace(tracer);
        self.resolve.trace(tracer);
        self.reject.trace(tracer);
    }
}

/// 把返回 promise 的同步步骤包装成“异常转 rejected promise”的形式。
pub fn with_promise_result<'js>(
    ctx: &Ctx<'js>,
    f: impl FnOnce() -> Result<Promise<'js>>,
) -> Result<Promise<'js>> {
    match f() {
        Ok(value) => Ok(value),
        Err(Error::Exception) => promise_rejected_catch(ctx, &*PromisePrimordials::get(ctx)?),
        Err(err) => Err(err),
    }
}
