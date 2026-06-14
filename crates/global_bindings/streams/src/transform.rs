use rquickjs::{
    function::{Opt, This},
    Class, Ctx, Error, Function, Object, Promise, Result, Value,
};

use crate::{
    readable::{ReadableStream, ReadableStreamDefaultController},
    utils::{
        get_optional_function, handle_maybe_promise, reject_later, resolve_later, throw_type_error,
    },
    writable::{WritableStream, WritableStreamDefaultController},
};

fn bind_method<'js>(
    ctx: &Ctx<'js>,
    func: &Function<'js>,
    this: &Class<'js, TransformStream<'js>>,
) -> Result<Function<'js>> {
    let binder: Function = ctx.eval("(function(f,o){ return f.bind(o); })")?;
    binder.call((func.clone(), this.clone()))
}

// =============================================================================
// TransformStreamDefaultController
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct TransformStreamDefaultController<'js> {
    stream: Class<'js, TransformStream<'js>>,
}

impl<'js> TransformStreamDefaultController<'js> {
    pub(crate) fn new(stream: Class<'js, TransformStream<'js>>) -> Self {
        Self { stream }
    }

    pub(crate) fn error_on(
        stream: &Class<'js, TransformStream<'js>>,
        ctx: &Ctx<'js>,
        err: Value<'js>,
    ) -> Result<()> {
        let (readable_controller, writable_controller) = {
            let s = stream.try_borrow()?;
            (s.readable_controller.clone(), s.writable_controller.clone())
        };
        if let Some(rc) = readable_controller {
            let _ = rc.try_borrow_mut()?.error(ctx.clone(), err.clone());
        }
        if let Some(wc) = writable_controller {
            let _ = wc.try_borrow_mut()?.error(ctx.clone(), err.clone());
        }
        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> TransformStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "TransformStreamDefaultController cannot be constructed directly",
        ))
    }

    fn enqueue(&mut self, ctx: Ctx<'js>, chunk: Value<'js>) -> Result<()> {
        if let Some(rc) = self.stream.try_borrow()?.readable_controller.clone() {
            rc.try_borrow_mut()?.enqueue(ctx, chunk)?;
        }
        Ok(())
    }

    fn terminate(&mut self, ctx: Ctx<'js>) -> Result<()> {
        if let Some(rc) = self.stream.try_borrow()?.readable_controller.clone() {
            rc.try_borrow_mut()?.close(ctx)?;
        }
        Ok(())
    }

    fn error(&mut self, ctx: Ctx<'js>, err: Value<'js>) -> Result<()> {
        Self::error_on(&self.stream, &ctx, err)
    }
}

// =============================================================================
// TransformStream
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct TransformStream<'js> {
    readable_controller: Option<Class<'js, ReadableStreamDefaultController<'js>>>,
    writable_controller: Option<Class<'js, WritableStreamDefaultController<'js>>>,
    controller: Option<Class<'js, TransformStreamDefaultController<'js>>>,
    transformer: Object<'js>,
    readable: Option<Class<'js, ReadableStream<'js>>>,
    writable: Option<Class<'js, WritableStream<'js>>>,
}

impl<'js> TransformStream<'js> {
    fn close_readable_side(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        if let Some(rc) = stream.try_borrow()?.readable_controller.clone() {
            rc.try_borrow_mut()?.close(ctx.clone())?;
        }
        Ok(())
    }

    fn relieve_backpressure(_stream: &Class<'js, Self>, _ctx: &Ctx<'js>) -> Result<()> {
        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> TransformStream<'js> {
    #[qjs(constructor)]
    fn new(
        ctx: Ctx<'js>,
        transformer: Opt<Object<'js>>,
        writable_strategy: Opt<Object<'js>>,
        readable_strategy: Opt<Object<'js>>,
    ) -> Result<Class<'js, Self>> {
        let transformer = transformer
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
        let writable_strategy = writable_strategy
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
        let readable_strategy = readable_strategy
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());

        let stream = Class::instance(
            ctx.clone(),
            Self {
                readable_controller: None,
                writable_controller: None,
                controller: None,
                transformer: transformer.clone(),
                readable: None,
                writable: None,
            },
        )?;

        let controller = Class::instance(
            ctx.clone(),
            TransformStreamDefaultController::new(stream.clone()),
        )?;
        stream.try_borrow_mut()?.controller = Some(controller.clone());

        // Readable side: closures are bound to the TransformStream instance so
        // they do not capture any JS values, avoiding hidden reference cycles.
        let source = Object::new(ctx.clone())?;
        let start = Function::new(
            ctx.clone(),
            move |_ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  ctrl: Class<'js, ReadableStreamDefaultController<'js>>|
                  -> Result<()> {
                this.0.try_borrow_mut()?.readable_controller = Some(ctrl);
                Ok(())
            },
        )?;
        source.set("start", bind_method(&ctx, &start, &stream)?)?;

        let pull = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  _ctrl: Class<'js, ReadableStreamDefaultController<'js>>|
                  -> Result<()> {
                TransformStream::relieve_backpressure(&this.0, &ctx)?;
                Ok(())
            },
        )?;
        source.set("pull", bind_method(&ctx, &pull, &stream)?)?;

        let cancel = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  reason: Opt<Value<'js>>|
                  -> Result<Value<'js>> {
                let reason = reason
                    .0
                    .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
                TransformStreamDefaultController::error_on(&this.0, &ctx, reason)?;
                Ok(Value::new_undefined(ctx))
            },
        )?;
        source.set("cancel", bind_method(&ctx, &cancel, &stream)?)?;

        let readable_ctor: Function = ctx.globals().get("ReadableStream")?;
        let readable_value: Value = readable_ctor.call((source, readable_strategy))?;
        let readable = Class::<ReadableStream>::from_value(&readable_value)?;
        stream.try_borrow_mut()?.readable = Some(readable);

        // Writable side
        let sink = Object::new(ctx.clone())?;
        let start = Function::new(
            ctx.clone(),
            move |_ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  ctrl: Class<'js, WritableStreamDefaultController<'js>>|
                  -> Result<()> {
                this.0.try_borrow_mut()?.writable_controller = Some(ctrl);
                Ok(())
            },
        )?;
        sink.set("start", bind_method(&ctx, &start, &stream)?)?;

        let write = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  chunk: Value<'js>,
                  _ctrl: Class<'js, WritableStreamDefaultController<'js>>|
                  -> Result<Value<'js>> {
                let (transformer, ts_controller, readable_controller) = {
                    let s = this.0.try_borrow()?;
                    (
                        s.transformer.clone(),
                        s.controller.clone().unwrap(),
                        s.readable_controller.clone(),
                    )
                };
                let transform = get_optional_function(&transformer, "transform")?;
                let (promise, resolve, reject) = Promise::new(&ctx)?;

                if let Some(transform) = transform {
                    match transform.call((chunk, ts_controller)) {
                        Ok(result) => {
                            let stream_ref = this.0.clone();
                            let stream_ref_err = stream_ref.clone();
                            handle_maybe_promise(
                                &ctx,
                                result,
                                move |ctx| {
                                    resolve_later(&ctx, resolve.clone(), ())?;
                                    Ok(())
                                },
                                move |ctx, err| {
                                    let _ = TransformStreamDefaultController::error_on(
                                        &stream_ref_err,
                                        &ctx,
                                        err.clone(),
                                    );
                                    Ok(())
                                },
                            )?;
                        }
                        Err(err) => {
                            let mut err_value = ctx.catch();
                            if err_value.is_undefined() {
                                throw_type_error(&ctx, &err.to_string());
                                err_value = ctx.catch();
                            }
                            let _ = TransformStreamDefaultController::error_on(
                                &this.0,
                                &ctx,
                                err_value.clone(),
                            );
                            reject_later(&ctx, reject, err_value)?;
                        }
                    }
                } else {
                    if let Some(rc) = readable_controller {
                        rc.try_borrow_mut()?.enqueue(ctx.clone(), chunk)?;
                    }
                    resolve_later(&ctx, resolve, ())?;
                }
                Ok(promise.into_inner().into_value())
            },
        )?;
        sink.set("write", bind_method(&ctx, &write, &stream)?)?;

        let close = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  _ctrl: Class<'js, WritableStreamDefaultController<'js>>|
                  -> Result<Value<'js>> {
                let (transformer, ts_controller) = {
                    let s = this.0.try_borrow()?;
                    (s.transformer.clone(), s.controller.clone().unwrap())
                };
                let flush = get_optional_function(&transformer, "flush")?;
                let (promise, resolve, reject) = Promise::new(&ctx)?;

                if let Some(flush) = flush {
                    match flush.call((ts_controller,)) {
                        Ok(result) => {
                            let stream_ref_ok = this.0.clone();
                            let stream_ref_err = this.0.clone();
                            handle_maybe_promise(
                                &ctx,
                                result,
                                move |_ctx| {
                                    TransformStream::close_readable_side(&stream_ref_ok, &_ctx)?;
                                    resolve_later(&_ctx, resolve.clone(), ())?;
                                    Ok(())
                                },
                                move |_ctx, err| {
                                    TransformStreamDefaultController::error_on(
                                        &stream_ref_err,
                                        &_ctx,
                                        err.clone(),
                                    )?;
                                    reject_later(&_ctx, reject.clone(), err)?;
                                    Ok(())
                                },
                            )?;
                        }
                        Err(err) => {
                            let mut err_value = ctx.catch();
                            if err_value.is_undefined() {
                                throw_type_error(&ctx, &err.to_string());
                                err_value = ctx.catch();
                            }
                            let _ = TransformStreamDefaultController::error_on(
                                &this.0,
                                &ctx,
                                err_value.clone(),
                            );
                            reject_later(&ctx, reject, err_value)?;
                        }
                    }
                } else {
                    TransformStream::close_readable_side(&this.0, &ctx)?;
                    resolve_later(&ctx, resolve, ())?;
                }
                Ok(promise.into_inner().into_value())
            },
        )?;
        sink.set("close", bind_method(&ctx, &close, &stream)?)?;

        let abort = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>,
                  this: This<Class<'js, TransformStream<'js>>>,
                  reason: Opt<Value<'js>>|
                  -> Result<Value<'js>> {
                let reason = reason
                    .0
                    .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
                TransformStreamDefaultController::error_on(&this.0, &ctx, reason)?;
                Ok(Value::new_undefined(ctx))
            },
        )?;
        sink.set("abort", bind_method(&ctx, &abort, &stream)?)?;

        let writable_ctor: Function = ctx.globals().get("WritableStream")?;
        let writable_value: Value = writable_ctor.call((sink, writable_strategy))?;
        let writable = Class::<WritableStream>::from_value(&writable_value)?;
        stream.try_borrow_mut()?.writable = Some(writable);

        Ok(stream)
    }

    #[qjs(get)]
    fn readable(&self) -> Result<Class<'js, ReadableStream<'js>>> {
        self.readable.clone().ok_or(Error::InvalidClass {
            class: "TransformStream",
            message: "readable side not initialized".to_string(),
        })
    }

    #[qjs(get)]
    fn writable(&self) -> Result<Class<'js, WritableStream<'js>>> {
        self.writable.clone().ok_or(Error::InvalidClass {
            class: "TransformStream",
            message: "writable side not initialized".to_string(),
        })
    }
}
