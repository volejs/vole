use rquickjs::{
    class::{Trace, Tracer},
    function::{Constructor, This},
    Ctx, Exception, Function, JsLifetime, Object, Result, Value,
};

/// Rust 侧的 `AbortSignal` 引用适配器。
///
/// 这个类型不是第二套 Abort 实现，它只持有已经存在于 JS 世界里的
/// `AbortSignal` 对象，并通过 Web API 暴露的属性和方法访问它。
///
/// 设计用途：
/// - 让 fetch、streams、timers 等底层 Rust 实现可以统一接收 JS 传入的
///   `AbortSignal`。
/// - 避免每个 binding crate 都复制一份 `AbortSignal` 包装逻辑。
/// - 保持 Abort 状态、reason、事件监听全部归属于 `vole-global-events`
///   暴露的 JS 对象，避免出现多套状态源。
///
/// 注意事项：
/// - 这里不校验对象是否真的是 `AbortSignal` 实例，只要求它是对象并具有
///   Web API 形态；这符合 JS duck typing 的使用方式，也方便跨 realm 或
///   polyfill 场景。
/// - 监听 abort 事件依赖对象上的 `addEventListener/removeEventListener`。
/// - Rust 异步任务保存该引用时必须保持 JS lifetime/trace 正确，避免回调或
///   reason 被 GC 提前回收。
#[derive(Clone)]
pub struct AbortSignalRef<'js> {
    object: Object<'js>,
}

unsafe impl<'js> JsLifetime<'js> for AbortSignalRef<'js> {
    type Changed<'to> = AbortSignalRef<'to>;
}

impl<'js> Trace<'js> for AbortSignalRef<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.object.trace(tracer);
    }
}

impl<'js> AbortSignalRef<'js> {
    /// 从任意 JS 值构造 AbortSignal 引用。
    ///
    /// Web API 中许多参数只要求 `{ signal }` 是一个对象；具体状态通过
    /// `aborted/reason/addEventListener` 读取，因此这里保持轻量校验。
    pub fn from_value(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let object = value
            .as_object()
            .cloned()
            .ok_or_else(|| Exception::throw_type(ctx, "Invalid signal argument"))?;
        Ok(Self { object })
    }

    /// 读取 `signal.aborted`。
    ///
    /// 缺失该属性时返回 `false`，用于容忍不完整的用户对象；真实的
    /// `AbortSignal` 会由事件 binding 保证该属性存在。
    pub fn aborted(&self) -> Result<bool> {
        self.object.get("aborted").or(Ok(false))
    }

    /// 读取 `signal.reason`。
    ///
    /// 缺失该属性时返回 `undefined`。这比抛错更适合底层取消流程，因为没有
    /// reason 的取消可以继续按默认 AbortError 语义处理。
    pub fn reason(&self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.object
            .get("reason")
            .or_else(|_| Ok(Value::new_undefined(ctx.clone())))
    }

    /// 注册一次 abort 事件监听。
    ///
    /// 调用方负责持有 `listener`，以便后续可用同一个函数对象移除监听。
    pub fn add_abort_listener(&self, listener: Function<'js>) -> Result<()> {
        let add_event_listener: Function = self.object.get("addEventListener")?;
        add_event_listener.call::<_, ()>((This(self.object.clone()), "abort", listener))
    }

    /// 移除之前注册的 abort 事件监听。
    ///
    /// 必须传入注册时同一个 `Function` 对象；否则 Web API 语义下不会移除
    /// 原监听。
    pub fn remove_abort_listener(&self, listener: Function<'js>) -> Result<()> {
        let remove_event_listener: Function = self.object.get("removeEventListener")?;
        remove_event_listener.call::<_, ()>((This(self.object.clone()), "abort", listener))
    }
}

/// Rust 侧的 `AbortController` 引用适配器。
///
/// 这个类型通过全局 `AbortController` 构造 JS 对象，并只调用其公开 Web API。
/// 底层 Rust 代码需要主动取消内部操作时使用它，例如 streams pipe 过程中为
/// 组合取消链路创建内部 controller。
#[derive(Clone)]
pub struct AbortControllerRef<'js> {
    object: Object<'js>,
}

unsafe impl<'js> JsLifetime<'js> for AbortControllerRef<'js> {
    type Changed<'to> = AbortControllerRef<'to>;
}

impl<'js> Trace<'js> for AbortControllerRef<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.object.trace(tracer);
    }
}

impl<'js> AbortControllerRef<'js> {
    /// 使用全局 `AbortController` 构造一个新的 controller。
    ///
    /// 该方法要求 `vole-global-events` 已经完成初始化，否则全局构造器不存在。
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        let ctor: Constructor = ctx.globals().get("AbortController")?;
        Ok(Self {
            object: ctor.construct(())?,
        })
    }

    /// 读取 `controller.signal`。
    ///
    /// 缺失时返回 `undefined`，保持和旧 streams 适配层一致，避免底层取消辅助
    /// 代码在非标准对象上立即崩溃。
    pub fn signal(&self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.object
            .get("signal")
            .or_else(|_| Ok(Value::new_undefined(ctx.clone())))
    }

    /// 调用 `controller.abort(reason?)`。
    pub fn abort(&self, reason: Option<Value<'js>>) -> Result<()> {
        let abort: Function = self.object.get("abort")?;
        match reason {
            Some(reason) => abort.call::<_, ()>((This(self.object.clone()), reason)),
            None => abort.call::<_, ()>((This(self.object.clone()),)),
        }
    }
}
