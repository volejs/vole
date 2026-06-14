use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Array, Class, Constructor, Ctx, Function, IntoJs, Null, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct MessageEvent<'js> {
    event_type: String,
    data: Value<'js>,
    origin: String,
    last_event_id: String,
    source: Value<'js>,
    ports: Value<'js>,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    default_prevented: bool,
    pub(crate) stop_propagation: bool,
    pub(crate) stop_immediate_propagation: bool,
    pub(crate) target: Option<Value<'js>>,
    pub(crate) current_target: Option<Value<'js>>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for MessageEvent<'js> {
    type Changed<'to> = MessageEvent<'to>;
}

impl<'js> Trace<'js> for MessageEvent<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.data.trace(tracer);
        self.source.trace(tracer);
        self.ports.trace(tracer);
        if let Some(target) = &self.target {
            target.trace(tracer);
        }
        if let Some(current_target) = &self.current_target {
            current_target.trace(tracer);
        }
    }
}

impl<'js> rquickjs::IntoJs<'js> for MessageEvent<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn message_event_ctor<'js>(
    _ctx: Ctx<'js>,
    event_type: String,
    options: rquickjs::function::Opt<Value<'js>>,
) -> Result<MessageEvent<'js>> {
    let mut data = Value::new_undefined(_ctx.clone());
    let mut origin = String::new();
    let mut last_event_id = String::new();
    let mut source = Value::new_undefined(_ctx.clone());
    let mut ports = Value::new_undefined(_ctx.clone());
    let mut bubbles = false;
    let mut cancelable = false;
    let mut composed = false;

    if let Some(options) = options.0 {
        if let Some(options) = options.as_object() {
            if let Ok(v) = options.get::<&str, Value>("data") {
                data = v;
            }
            if let Ok(v) = options.get::<&str, String>("origin") {
                origin = v;
            }
            if let Ok(v) = options.get::<&str, String>("lastEventId") {
                last_event_id = v;
            }
            if let Ok(v) = options.get::<&str, Value>("source") {
                source = v;
            }
            if let Ok(arr) = options.get::<&str, Array>("ports") {
                ports = arr.into_value();
            } else {
                ports = Array::new(_ctx.clone())?.into_value();
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

    Ok(MessageEvent {
        event_type,
        data,
        origin,
        last_event_id,
        source,
        ports,
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

fn message_event_get_type<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<String> {
    Ok(this.try_borrow()?.event_type.clone())
}

fn message_event_get_data<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.data.clone())
}

fn message_event_get_origin<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<String> {
    Ok(this.try_borrow()?.origin.clone())
}

fn message_event_get_last_event_id<'js>(
    this: This<Class<'js, MessageEvent<'js>>>,
) -> Result<String> {
    Ok(this.try_borrow()?.last_event_id.clone())
}

fn message_event_get_source<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.source.clone())
}

fn message_event_get_ports<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.ports.clone())
}

fn message_event_get_bubbles<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.bubbles)
}

fn message_event_get_cancelable<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.cancelable)
}

fn message_event_get_composed<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<bool> {
    Ok(this.try_borrow()?.composed)
}

fn message_event_get_default_prevented<'js>(
    this: This<Class<'js, MessageEvent<'js>>>,
) -> Result<bool> {
    Ok(this.try_borrow()?.default_prevented)
}

fn message_event_get_target<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<Value<'js>> {
    let target = this.try_borrow()?.target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn message_event_get_current_target<'js>(
    this: This<Class<'js, MessageEvent<'js>>>,
) -> Result<Value<'js>> {
    let target = this.try_borrow()?.current_target.clone();
    match target {
        Some(v) => Ok(v),
        None => Null.into_js(&this.as_object().unwrap().ctx()),
    }
}

fn message_event_prevent_default<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    if ev.cancelable {
        ev.default_prevented = true;
    }
    Ok(())
}

fn message_event_stop_propagation<'js>(this: This<Class<'js, MessageEvent<'js>>>) -> Result<()> {
    this.try_borrow_mut()?.stop_propagation = true;
    Ok(())
}

fn message_event_stop_immediate_propagation<'js>(
    this: This<Class<'js, MessageEvent<'js>>>,
) -> Result<()> {
    let mut ev = this.try_borrow_mut()?;
    ev.stop_propagation = true;
    ev.stop_immediate_propagation = true;
    Ok(())
}

fn message_event_composed_path<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, MessageEvent<'js>>>,
) -> Result<Array<'js>> {
    Array::new(ctx)
}

impl<'js> JsClass<'js> for MessageEvent<'js> {
    const NAME: &'static str = "MessageEvent";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("type", Function::new(ctx.clone(), message_event_get_type)?)?;
        proto.set("data", Function::new(ctx.clone(), message_event_get_data)?)?;
        proto.set(
            "origin",
            Function::new(ctx.clone(), message_event_get_origin)?,
        )?;
        proto.set(
            "lastEventId",
            Function::new(ctx.clone(), message_event_get_last_event_id)?,
        )?;
        proto.set(
            "source",
            Function::new(ctx.clone(), message_event_get_source)?,
        )?;
        proto.set(
            "ports",
            Function::new(ctx.clone(), message_event_get_ports)?,
        )?;
        proto.set(
            "bubbles",
            Function::new(ctx.clone(), message_event_get_bubbles)?,
        )?;
        proto.set(
            "cancelable",
            Function::new(ctx.clone(), message_event_get_cancelable)?,
        )?;
        proto.set(
            "composed",
            Function::new(ctx.clone(), message_event_get_composed)?,
        )?;
        proto.set(
            "defaultPrevented",
            Function::new(ctx.clone(), message_event_get_default_prevented)?,
        )?;
        proto.set(
            "target",
            Function::new(ctx.clone(), message_event_get_target)?,
        )?;
        proto.set(
            "currentTarget",
            Function::new(ctx.clone(), message_event_get_current_target)?,
        )?;
        proto.set(
            "preventDefault",
            Function::new(ctx.clone(), message_event_prevent_default)?,
        )?;
        proto.set(
            "stopPropagation",
            Function::new(ctx.clone(), message_event_stop_propagation)?,
        )?;
        proto.set(
            "stopImmediatePropagation",
            Function::new(ctx.clone(), message_event_stop_immediate_propagation)?,
        )?;
        proto.set(
            "composedPath",
            Function::new(ctx.clone(), message_event_composed_path)?,
        )?;

        let event_proto: Object<'js> = ctx
            .globals()
            .get::<&str, Function>("Event")?
            .get("prototype")?;
        proto.set_prototype(Some(&event_proto))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<MessageEvent, _, _>(ctx.clone(), message_event_ctor)?;
        Ok(Some(constr))
    }
}
