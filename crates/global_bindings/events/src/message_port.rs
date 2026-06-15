use rquickjs::{
    Array, Class, Constructor, Ctx, Function, Object, Result, Value,
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
};

#[derive(Clone, Debug)]
pub struct MessagePort;

unsafe impl<'js> rquickjs::JsLifetime<'js> for MessagePort {
    type Changed<'to> = MessagePort;
}

impl<'js> Trace<'js> for MessagePort {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for MessagePort {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn message_port_ctor<'js>(_ctx: Ctx<'js>) -> Result<MessagePort> {
    Ok(MessagePort)
}

fn message_port_post_message<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, MessagePort>>,
    data: Value<'js>,
    transfer: Opt<Value<'js>>,
) -> Result<()> {
    let obj = this.as_object().expect("This<Class> should have an object");
    if obj.get::<_, bool>("_closed").unwrap_or(false) {
        return Ok(());
    }
    let pair: Value<'js> = obj.get("_pair")?;
    if pair.is_null() || pair.is_undefined() {
        return Ok(());
    }
    let pair_obj = Object::from_value(pair)?;
    if pair_obj.get::<_, bool>("_closed").unwrap_or(false) {
        return Ok(());
    }

    let data = match ctx.globals().get::<_, Function>("structuredClone") {
        Ok(structured_clone) => {
            if let Some(transfer) = transfer.0 {
                let options = Object::new(ctx.clone())?;
                options.set("transfer", transfer)?;
                structured_clone.call((data, options))?
            } else {
                structured_clone.call((data,))?
            }
        }
        Err(_) => data,
    };

    let event = Object::new(ctx.clone())?;
    event.set("type", "message")?;
    event.set("data", data)?;
    event.set("origin", "")?;
    event.set("lastEventId", "")?;
    event.set("source", obj.clone())?;
    event.set("ports", Array::new(ctx.clone())?)?;

    ctx.clone().spawn(async move {
        tokio::task::yield_now().await;
        let _ = crate::event_target::dispatch_event_on_object(&ctx, &pair_obj, &event);
    });
    Ok(())
}

fn message_port_start(_this: This<Class<'_, MessagePort>>) -> Result<()> {
    Ok(())
}

fn message_port_close(this: This<Class<'_, MessagePort>>) -> Result<()> {
    let obj = this.as_object().expect("This<Class> should have an object");
    obj.set("_closed", true)?;
    let pair: Value<'_> = obj.get("_pair")?;
    if !pair.is_null() && !pair.is_undefined() {
        if let Ok(pair_obj) = Object::from_value(pair) {
            let _ = pair_obj.set("_closed", true);
        }
    }
    Ok(())
}

impl<'js> JsClass<'js> for MessagePort {
    const NAME: &'static str = "MessagePort";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "postMessage",
            Function::new(ctx.clone(), message_port_post_message)?,
        )?;
        proto.set("start", Function::new(ctx.clone(), message_port_start)?)?;
        proto.set("close", Function::new(ctx.clone(), message_port_close)?)?;

        let event_target_proto: Object<'js> = ctx
            .globals()
            .get::<&str, Function>("EventTarget")?
            .get("prototype")?;
        proto.set_prototype(Some(&event_target_proto))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<MessagePort, _, _>(ctx.clone(), message_port_ctor)?;
        Ok(Some(constr))
    }
}
