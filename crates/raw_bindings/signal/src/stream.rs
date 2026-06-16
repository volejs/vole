use std::sync::Arc;

use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace, Tracer, Writable},
    function::{Constructor, Opt, This},
    Class, Ctx, Exception, FromJs, Function, IntoJs, Object, Promise, Result, Value,
};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

use crate::registry::{create_subscription, remove_subscription};

#[derive(Clone)]
pub struct SignalStream {
    id: Option<u64>,
    receiver: Option<Arc<AsyncMutex<mpsc::UnboundedReceiver<String>>>>,
    closed: bool,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for SignalStream {
    type Changed<'to> = SignalStream;
}

impl<'js> Trace<'js> for SignalStream {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for SignalStream {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

impl Drop for SignalStream {
    fn drop(&mut self) {
        if !self.closed {
            self.closed = true;
            if let Some(id) = self.id.take() {
                remove_subscription(id);
            }
        }
    }
}

struct SignalIteratorResult {
    value: Option<String>,
    done: bool,
}

impl<'js> IntoJs<'js> for SignalIteratorResult {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        let object = Object::new(ctx.clone())?;
        match self.value {
            Some(value) => object.set("value", value)?,
            None => object.set("value", Value::new_undefined(ctx.clone()))?,
        }
        object.set("done", self.done)?;
        Ok(object.into_value())
    }
}

fn signal_stream_new<'js>(ctx: &Ctx<'js>, signals: Vec<String>) -> Result<SignalStream> {
    let (id, receiver) = create_subscription(ctx, signals)?;
    Ok(SignalStream {
        id: Some(id),
        receiver: Some(receiver),
        closed: false,
    })
}

fn signal_vec_from_value<'js>(ctx: &Ctx<'js>, value: Option<Value<'js>>) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_undefined() {
        return Ok(Vec::new());
    }
    Vec::<String>::from_js(ctx, value)
}

fn dispatch_signal_event<'js>(ctx: &Ctx<'js>, target: &Object<'js>, signal: &str) -> Result<()> {
    let options = Object::new(ctx.clone())?;
    options.set("detail", signal)?;

    let event: Object<'js> =
        if let Ok(custom_event_ctor) = ctx.globals().get::<_, Function<'js>>("CustomEvent") {
            custom_event_ctor.call(("signal", options))?
        } else {
            let event_ctor: Function<'js> = ctx.globals().get("Event")?;
            let event: Object<'js> = event_ctor.call(("signal",))?;
            event.set("detail", signal)?;
            event
        };

    let dispatch_event: Function<'js> = target.get("dispatchEvent")?;
    dispatch_event.call::<_, ()>((This(target.clone()), event))
}

fn signal_stream_next<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, SignalStream>>,
) -> Result<Promise<'js>> {
    let receiver = {
        let stream = this.0.try_borrow()?;
        if stream.closed {
            let (promise, resolve, _reject) = Promise::new(&ctx)?;
            resolve.call::<_, ()>((SignalIteratorResult {
                value: None,
                done: true,
            },))?;
            return Ok(promise);
        }
        stream
            .receiver
            .clone()
            .ok_or_else(|| Exception::throw_type(&ctx, "SignalStream is not initialized"))?
    };
    let target = this
        .as_object()
        .cloned()
        .ok_or_else(|| Exception::throw_type(&ctx, "SignalStream receiver is invalid"))?;

    let (promise, resolve, reject) = Promise::new(&ctx)?;
    let ctx_for_resolve = ctx.clone();
    ctx.spawn(async move {
        let mut lock = receiver.lock().await;
        let result = match lock.recv().await {
            Some(signal) => {
                if let Err(err) = dispatch_signal_event(&ctx_for_resolve, &target, &signal) {
                    let _ = reject.call::<_, ()>((err.to_string(),));
                    return;
                }
                SignalIteratorResult {
                    value: Some(signal),
                    done: false,
                }
            }
            None => SignalIteratorResult {
                value: None,
                done: true,
            },
        };
        let _ = resolve.call::<_, ()>((result,));
    });

    Ok(promise)
}

fn signal_stream_async_iterator<'js>(this: This<Value<'js>>) -> Result<Value<'js>> {
    Ok(this.0)
}

fn signal_stream_close<'js>(this: This<Class<'js, SignalStream>>) -> Result<()> {
    let mut stream = this.0.try_borrow_mut()?;
    if stream.closed {
        return Ok(());
    }

    stream.closed = true;
    if let Some(id) = stream.id.take() {
        remove_subscription(id);
    }
    stream.receiver = None;
    Ok(())
}

fn signal_stream_is_closed<'js>(this: This<Class<'js, SignalStream>>) -> Result<bool> {
    Ok(this.0.try_borrow()?.closed)
}

impl<'js> JsClass<'js> for SignalStream {
    const NAME: &'static str = "SignalStream";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("next", Function::new(ctx.clone(), signal_stream_next)?)?;
        proto.set("close", Function::new(ctx.clone(), signal_stream_close)?)?;
        proto.set(
            "isClosed",
            Function::new(ctx.clone(), signal_stream_is_closed)?,
        )?;
        proto.set(
            PredefinedAtom::SymbolAsyncIterator,
            Function::new(ctx.clone(), signal_stream_async_iterator)?,
        )?;

        if let Ok(event_target) = ctx.globals().get::<_, Function<'js>>("EventTarget") {
            if let Ok(event_target_proto) = event_target.get::<_, Object<'js>>("prototype") {
                proto.set_prototype(Some(&event_target_proto))?;
            }
        }

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(ctx: Ctx<'js>, signals: Opt<Value<'js>>) -> Result<SignalStream> {
            let signals = signal_vec_from_value(&ctx, signals.0)?;
            signal_stream_new(&ctx, signals)
        }

        let constr = Constructor::new_class::<SignalStream, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}
