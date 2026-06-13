use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Class, Constructor, Ctx, Function, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct AbortController<'js> {
    signal: Value<'js>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for AbortController<'js> {
    type Changed<'to> = AbortController<'to>;
}

impl<'js> Trace<'js> for AbortController<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.signal.trace(tracer);
    }
}

impl<'js> rquickjs::IntoJs<'js> for AbortController<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn abort_controller_ctor<'js>(ctx: Ctx<'js>) -> Result<AbortController<'js>> {
    let signal = Class::instance(ctx.clone(), crate::abort_signal::AbortSignal {
        aborted: false,
        reason: None,
    })?;
    Ok(AbortController {
        signal: signal.into_value(),
    })
}

fn abort_controller_get_signal<'js>(
    this: This<Class<'js, AbortController<'js>>>,
) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.signal.clone())
}

fn abort_controller_abort<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, AbortController<'js>>>,
    reason: Opt<Value<'js>>,
) -> Result<()> {
    let ctrl = this.try_borrow_mut()?;
    let signal_obj = Object::from_value(ctrl.signal.clone())?;
    if let Some(signal) = Class::<crate::abort_signal::AbortSignal>::from_object(&signal_obj) {
        let mut sig = signal.try_borrow_mut()?;
        sig.aborted = true;
        sig.reason = reason.0.clone();
    }

    let event = Object::new(ctx.clone())?;
    event.set("type", "abort")?;
    crate::event_target::dispatch_event_on_object(&ctx, &signal_obj, &event)?;
    Ok(())
}

impl<'js> JsClass<'js> for AbortController<'js> {
    const NAME: &'static str = "AbortController";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "signal",
            Function::new(ctx.clone(), abort_controller_get_signal)?,
        )?;
        proto.set("abort", Function::new(ctx.clone(), abort_controller_abort)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr =
            Constructor::new_class::<AbortController, _, _>(ctx.clone(), abort_controller_ctor)?;
        Ok(Some(constr))
    }
}
