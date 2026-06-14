use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Class, Constructor, Ctx, Function, Object, Result, Value,
};
use vole_core::error::{dom_exception_name, dom_exception_value};

#[derive(Clone, Debug)]
pub struct AbortSignal<'js> {
    pub(crate) aborted: bool,
    pub(crate) reason: Option<Value<'js>>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for AbortSignal<'js> {
    type Changed<'to> = AbortSignal<'to>;
}

impl<'js> Trace<'js> for AbortSignal<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Some(reason) = &self.reason {
            reason.trace(tracer);
        }
    }
}

impl<'js> rquickjs::IntoJs<'js> for AbortSignal<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn abort_signal_ctor<'js>(ctx: Ctx<'js>) -> Result<AbortSignal<'js>> {
    Err(rquickjs::Exception::throw_type(&ctx, "Illegal constructor"))
}

fn abort_signal_get_aborted<'js>(this: This<Class<'js, AbortSignal<'js>>>) -> Result<bool> {
    let rust = this.try_borrow()?.aborted;
    if rust {
        return Ok(true);
    }
    let obj = this.as_object().expect("This<Class> should have an object");
    obj.get::<&str, bool>("_aborted").or(Ok(false))
}

fn abort_signal_get_reason<'js>(this: This<Class<'js, AbortSignal<'js>>>) -> Result<Value<'js>> {
    let reason = this.try_borrow()?.reason.clone();
    if let Some(v) = reason {
        return Ok(v);
    }
    let obj = this.as_object().expect("This<Class> should have an object");
    if let Ok(v) = obj.get::<&str, Value>("_reason") {
        return Ok(v);
    }
    Ok(Value::new_undefined(obj.ctx().clone()))
}

fn abort_signal_abort<'js>(ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<Value<'js>> {
    let reason = match reason.0 {
        Some(r) => r,
        None => dom_exception_value(
            &ctx,
            dom_exception_name::ABORT,
            "This operation was aborted",
        )?,
    };
    let signal = AbortSignal {
        aborted: true,
        reason: Some(reason.clone()),
    };
    let obj = Class::instance(ctx.clone(), signal)?;
    Ok(obj.into_value())
}

impl<'js> JsClass<'js> for AbortSignal<'js> {
    const NAME: &'static str = "AbortSignal";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "aborted",
            Function::new(ctx.clone(), abort_signal_get_aborted)?,
        )?;
        proto.set(
            "reason",
            Function::new(ctx.clone(), abort_signal_get_reason)?,
        )?;

        let event_target_proto: Object<'js> = ctx
            .globals()
            .get::<&str, Function>("EventTarget")?
            .get("prototype")?;
        proto.set_prototype(Some(&event_target_proto))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<AbortSignal, _, _>(ctx.clone(), abort_signal_ctor)?;
        let constr_obj = constr.clone();
        constr_obj.set("abort", Function::new(ctx.clone(), abort_signal_abort)?)?;
        Ok(Some(constr))
    }
}
