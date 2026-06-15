use rquickjs::{
    Array, Class, Constructor, Ctx, Function, IntoJs, Null, Object, Result, Value,
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
};

#[derive(Clone, Debug)]
pub struct CustomEvent<'js> {
    event_type: String,
    detail: Option<Value<'js>>,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    default_prevented: bool,
    pub(crate) stop_propagation: bool,
    pub(crate) stop_immediate_propagation: bool,
    pub(crate) target: Option<Value<'js>>,
    pub(crate) current_target: Option<Value<'js>>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for CustomEvent<'js> {
    type Changed<'to> = CustomEvent<'to>;
}

impl<'js> Trace<'js> for CustomEvent<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Some(detail) = &self.detail {
            detail.trace(tracer);
        }
        if let Some(target) = &self.target {
            target.trace(tracer);
        }
        if let Some(current_target) = &self.current_target {
            current_target.trace(tracer);
        }
    }
}

impl<'js> rquickjs::IntoJs<'js> for CustomEvent<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn custom_event_ctor<'js>(
    _ctx: Ctx<'js>,
    event_type: String,
    options: rquickjs::function::Opt<Value<'js>>,
) -> Result<CustomEvent<'js>> {
    let mut detail = None;
    let mut bubbles = false;
    let mut cancelable = false;
    let mut composed = false;
    if let Some(options) = options.0 {
        if let Some(options) = options.as_object() {
            if let Ok(v) = options.get::<&str, Value>("detail") {
                detail = Some(v);
            }
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
    Ok(CustomEvent {
        event_type,
        detail,
        bubbles,
        cancelable,
        composed,
        default_prevented: false,
        stop_propagation: false,
        stop_immediate_propagation: false,
        target: None,
        current_target: None,
    })
}

fn custom_event_get_type<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<String> {
    Ok(this.try_borrow()?.event_type.clone())
}

fn custom_event_get_detail<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<Value<'js>> {
    let detail = this.try_borrow()?.detail.clone();
    match detail {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn custom_event_get_bubbles<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.bubbles)
}

fn custom_event_get_cancelable<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.cancelable)
}

fn custom_event_get_composed<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.composed)
}

fn custom_event_get_default_prevented<'js>(
    this: This<Class<'js, CustomEvent<'js>>>,
) -> Result<bool> {
    Ok(this.try_borrow()?.default_prevented)
}

fn custom_event_get_target<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<Value<'js>> {
    let target = this.try_borrow()?.target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn custom_event_get_current_target<'js>(
    this: This<Class<'js, CustomEvent<'js>>>,
) -> Result<Value<'js>> {
    let target = this.try_borrow()?.current_target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn custom_event_prevent_default<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    if ev.cancelable {
        ev.default_prevented = true;
    }
    Ok(())
}

fn custom_event_stop_propagation<'js>(this: This<Class<'js, CustomEvent<'js>>>) -> Result<()> {
    this.try_borrow_mut()?.stop_propagation = true;
    Ok(())
}

fn custom_event_stop_immediate_propagation<'js>(
    this: This<Class<'js, CustomEvent<'js>>>,
) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    ev.stop_propagation = true;
    ev.stop_immediate_propagation = true;
    Ok(())
}

fn custom_event_composed_path<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, CustomEvent<'js>>>,
) -> Result<Array<'js>> {
    Array::new(ctx)
}

impl<'js> JsClass<'js> for CustomEvent<'js> {
    const NAME: &'static str = "CustomEvent";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("type", Function::new(ctx.clone(), custom_event_get_type)?)?;
        proto.set(
            "detail",
            Function::new(ctx.clone(), custom_event_get_detail)?,
        )?;
        proto.set(
            "bubbles",
            Function::new(ctx.clone(), custom_event_get_bubbles)?,
        )?;
        proto.set(
            "cancelable",
            Function::new(ctx.clone(), custom_event_get_cancelable)?,
        )?;
        proto.set(
            "composed",
            Function::new(ctx.clone(), custom_event_get_composed)?,
        )?;
        proto.set(
            "defaultPrevented",
            Function::new(ctx.clone(), custom_event_get_default_prevented)?,
        )?;
        proto.set(
            "target",
            Function::new(ctx.clone(), custom_event_get_target)?,
        )?;
        proto.set(
            "currentTarget",
            Function::new(ctx.clone(), custom_event_get_current_target)?,
        )?;
        proto.set(
            "preventDefault",
            Function::new(ctx.clone(), custom_event_prevent_default)?,
        )?;
        proto.set(
            "stopPropagation",
            Function::new(ctx.clone(), custom_event_stop_propagation)?,
        )?;
        proto.set(
            "stopImmediatePropagation",
            Function::new(ctx.clone(), custom_event_stop_immediate_propagation)?,
        )?;
        proto.set(
            "composedPath",
            Function::new(ctx.clone(), custom_event_composed_path)?,
        )?;

        let event_proto: Object<'js> = ctx
            .globals()
            .get::<&str, Function>("Event")?
            .get("prototype")?;
        proto.set_prototype(Some(&event_proto))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<CustomEvent, _, _>(ctx.clone(), custom_event_ctor)?;
        Ok(Some(constr))
    }
}
