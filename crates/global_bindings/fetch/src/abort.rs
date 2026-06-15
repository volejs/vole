use rquickjs::{
    class::{Trace, Tracer},
    Ctx, Exception, FromJs, Function, IntoJs, JsLifetime, Object, Result, Value,
};
use vole_core::mc_oneshot;
use vole_global_events::AbortSignalRef;

#[derive(Clone)]
pub(crate) struct AbortSignal<'js> {
    signal: AbortSignalRef<'js>,
    object: Object<'js>,
    pub(crate) sender: mc_oneshot::Sender<Value<'js>>,
    listener: Option<Function<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for AbortSignal<'js> {
    type Changed<'to> = AbortSignal<'to>;
}

impl<'js> Trace<'js> for AbortSignal<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.signal.trace(tracer);
        self.object.trace(tracer);
        self.sender.trace(tracer);
        if let Some(listener) = &self.listener {
            listener.trace(tracer);
        }
    }
}

impl<'js> FromJs<'js> for AbortSignal<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::from_value(ctx, value)
    }
}

impl<'js> IntoJs<'js> for AbortSignal<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.object.into_value())
    }
}

impl<'js> AbortSignal<'js> {
    pub(crate) fn from_value(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let object = value
            .as_object()
            .cloned()
            .ok_or_else(|| Exception::throw_type(ctx, "Invalid signal argument"))?;
        let signal = AbortSignalRef::from_value(ctx, object.clone().into_value())?;
        let (sender, _) = mc_oneshot::channel();

        let mut adapter = Self {
            signal,
            object,
            sender,
            listener: None,
        };

        if adapter.aborted()? {
            let reason = adapter.reason(ctx)?;
            adapter.sender.send(reason);
            return Ok(adapter);
        }

        let listener_sender = adapter.sender.clone();
        let listener_signal = adapter.signal.clone();
        let listener = Function::new(ctx.clone(), move |ctx: Ctx<'js>| -> Result<()> {
            let reason = listener_signal.reason(&ctx)?;
            listener_sender.send(reason);
            Ok(())
        })?;

        if adapter.signal.add_abort_listener(listener.clone()).is_ok() {
            adapter.listener = Some(listener);
        }

        Ok(adapter)
    }

    pub(crate) fn aborted(&self) -> Result<bool> {
        self.signal.aborted()
    }

    pub(crate) fn reason(&self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.signal.reason(ctx)
    }
}
