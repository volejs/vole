use rquickjs::{
    Array, Class, Constructor, Ctx, Function, IntoJs, Null, Object, Result, Value,
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
};
use vole_core::time::TimeOrigin;

#[derive(Clone, Debug)]
pub struct Event<'js> {
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    default_prevented: bool,
    time_stamp: f64,
    event_phase: u16,
    is_trusted: bool,
    pub(crate) stop_propagation: bool,
    pub(crate) stop_immediate_propagation: bool,
    pub(crate) target: Option<Value<'js>>,
    pub(crate) current_target: Option<Value<'js>>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for Event<'js> {
    type Changed<'to> = Event<'to>;
}

impl<'js> Trace<'js> for Event<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Some(target) = &self.target {
            target.trace(tracer);
        }
        if let Some(current_target) = &self.current_target {
            current_target.trace(tracer);
        }
    }
}

impl<'js> rquickjs::IntoJs<'js> for Event<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn event_ctor<'js>(
    ctx: Ctx<'js>,
    event_type: String,
    options: rquickjs::function::Opt<Value<'js>>,
) -> Result<Event<'js>> {
    let mut bubbles = false;
    let mut cancelable = false;
    let mut composed = false;
    if let Some(options) = options.0 {
        if let Some(options) = options.as_object() {
            if let Ok(v) = options.get::<&str, bool>("bubbles") {
                bubbles = v;
            }
            if let Ok(v) = options.get::<&str, bool>("cancelable") {
                cancelable = v;
            }
            if let Ok(v) = options.get::<&str, bool>("composed") {
                composed = v;
            }
        }
    }
    Ok(Event {
        event_type,
        bubbles,
        cancelable,
        composed,
        default_prevented: false,
        time_stamp: TimeOrigin::get_or_init(&ctx)?.now(),
        event_phase: 0,
        is_trusted: false,
        stop_propagation: false,
        stop_immediate_propagation: false,
        target: None,
        current_target: None,
    })
}

fn event_get_type<'js>(this: This<Class<'js, Event<'js>>>) -> Result<String> {
    Ok(this.try_borrow()?.event_type.clone())
}

fn event_get_bubbles<'js>(this: This<Class<'js, Event<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.bubbles)
}

fn event_get_cancelable<'js>(this: This<Class<'js, Event<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.cancelable)
}

fn event_get_composed<'js>(this: This<Class<'js, Event<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.composed)
}

fn event_get_default_prevented<'js>(this: This<Class<'js, Event<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.default_prevented)
}

fn event_get_time_stamp<'js>(this: This<Class<'js, Event<'js>>>) -> Result<f64> {
    Ok(this.try_borrow()?.time_stamp)
}

fn event_get_event_phase<'js>(this: This<Class<'js, Event<'js>>>) -> Result<u16> {
    Ok(this.try_borrow()?.event_phase)
}

fn event_get_is_trusted<'js>(this: This<Class<'js, Event<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.is_trusted)
}

fn event_get_target<'js>(this: This<Class<'js, Event<'js>>>) -> Result<Value<'js>> {
    let target = this.try_borrow()?.target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn event_get_current_target<'js>(this: This<Class<'js, Event<'js>>>) -> Result<Value<'js>> {
    let target = this.try_borrow()?.current_target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn event_prevent_default<'js>(this: This<Class<'js, Event<'js>>>) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    if ev.cancelable {
        ev.default_prevented = true;
    }
    Ok(())
}

fn event_stop_propagation<'js>(this: This<Class<'js, Event<'js>>>) -> Result<()> {
    this.try_borrow_mut()?.stop_propagation = true;
    Ok(())
}

fn event_stop_immediate_propagation<'js>(this: This<Class<'js, Event<'js>>>) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    ev.stop_propagation = true;
    ev.stop_immediate_propagation = true;
    Ok(())
}

fn event_composed_path<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, Event<'js>>>,
) -> Result<Array<'js>> {
    let array = Array::new(ctx)?;
    if let Some(target) = this.try_borrow()?.target.clone() {
        array.set(0, target)?;
    }
    Ok(array)
}

impl<'js> JsClass<'js> for Event<'js> {
    const NAME: &'static str = "Event";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("type", Function::new(ctx.clone(), event_get_type)?)?;
        proto.set("bubbles", Function::new(ctx.clone(), event_get_bubbles)?)?;
        proto.set(
            "cancelable",
            Function::new(ctx.clone(), event_get_cancelable)?,
        )?;
        proto.set("composed", Function::new(ctx.clone(), event_get_composed)?)?;
        proto.set(
            "defaultPrevented",
            Function::new(ctx.clone(), event_get_default_prevented)?,
        )?;
        proto.set(
            "timeStamp",
            Function::new(ctx.clone(), event_get_time_stamp)?,
        )?;
        proto.set(
            "eventPhase",
            Function::new(ctx.clone(), event_get_event_phase)?,
        )?;
        proto.set(
            "isTrusted",
            Function::new(ctx.clone(), event_get_is_trusted)?,
        )?;
        proto.set("target", Function::new(ctx.clone(), event_get_target)?)?;
        proto.set(
            "currentTarget",
            Function::new(ctx.clone(), event_get_current_target)?,
        )?;
        proto.set(
            "preventDefault",
            Function::new(ctx.clone(), event_prevent_default)?,
        )?;
        proto.set(
            "stopPropagation",
            Function::new(ctx.clone(), event_stop_propagation)?,
        )?;
        proto.set(
            "stopImmediatePropagation",
            Function::new(ctx.clone(), event_stop_immediate_propagation)?,
        )?;
        proto.set(
            "composedPath",
            Function::new(ctx.clone(), event_composed_path)?,
        )?;

        let event_target_proto: Object<'js> = ctx
            .globals()
            .get::<&str, Function>("EventTarget")?
            .get("prototype")?;
        proto.set_prototype(Some(&event_target_proto))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<Event, _, _>(ctx.clone(), event_ctor)?;
        Ok(Some(constr))
    }
}
