use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Class, Constructor, Ctx, Function, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct MessageChannel<'js> {
    port1: Value<'js>,
    port2: Value<'js>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for MessageChannel<'js> {
    type Changed<'to> = MessageChannel<'to>;
}

impl<'js> Trace<'js> for MessageChannel<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.port1.trace(tracer);
        self.port2.trace(tracer);
    }
}

impl<'js> rquickjs::IntoJs<'js> for MessageChannel<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn message_channel_ctor<'js>(ctx: Ctx<'js>) -> Result<MessageChannel<'js>> {
    let port1 = Class::instance(ctx.clone(), crate::message_port::MessagePort)?;
    let port2 = Class::instance(ctx.clone(), crate::message_port::MessagePort)?;

    let port1_value = port1.clone().into_value();
    let port2_value = port2.clone().into_value();

    let port1_obj = Object::from_value(port1_value.clone())?;
    let port2_obj = Object::from_value(port2_value.clone())?;
    port1_obj.set("_pair", port2_value.clone())?;
    port2_obj.set("_pair", port1_value.clone())?;

    Ok(MessageChannel {
        port1: port1_value,
        port2: port2_value,
    })
}

fn message_channel_get_port1<'js>(
    this: This<Class<'js, MessageChannel<'js>>>,
) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.port1.clone())
}

fn message_channel_get_port2<'js>(
    this: This<Class<'js, MessageChannel<'js>>>,
) -> Result<Value<'js>> {
    Ok(this.try_borrow()?.port2.clone())
}

impl<'js> JsClass<'js> for MessageChannel<'js> {
    const NAME: &'static str = "MessageChannel";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "port1",
            Function::new(ctx.clone(), message_channel_get_port1)?,
        )?;
        proto.set(
            "port2",
            Function::new(ctx.clone(), message_channel_get_port2)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr =
            Constructor::new_class::<MessageChannel, _, _>(ctx.clone(), message_channel_ctor)?;
        Ok(Some(constr))
    }
}
