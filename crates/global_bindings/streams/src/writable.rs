use rquickjs::{class::Trace, function::Opt, Class, Ctx, Function, Object, Promise, Result, Value};

use crate::utils::{
    get_optional_function, handle_maybe_promise, reject_later, resolve_later, resolved_promise,
    throw_type_error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WritableStreamState {
    Writable,
    Closed,
    Errored,
}

impl<'js> Trace<'js> for WritableStreamState {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[derive(rquickjs::JsLifetime, Trace)]
struct WriteRecord<'js> {
    chunk: Value<'js>,
    size: f64,
    resolve: Function<'js>,
    reject: Function<'js>,
}

// =============================================================================
// WritableStreamDefaultController
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct WritableStreamDefaultController<'js> {
    stream: Class<'js, WritableStream<'js>>,
    signal: Value<'js>,
    abort_controller: Object<'js>,
}

impl<'js> WritableStreamDefaultController<'js> {
    pub(crate) fn new(
        stream: Class<'js, WritableStream<'js>>,
        signal: Value<'js>,
        abort_controller: Object<'js>,
    ) -> Self {
        Self {
            stream,
            signal,
            abort_controller,
        }
    }

    pub(crate) fn abort_signal(&self, ctx: &Ctx<'js>, reason: Value<'js>) -> Result<()> {
        let helper: Function =
            ctx.eval("(function(controller, reason){ controller.abort(reason); })")?;
        let _: std::result::Result<(), rquickjs::Error> =
            helper.call((self.abort_controller.clone(), reason));
        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> WritableStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "WritableStreamDefaultController cannot be constructed directly",
        ))
    }

    #[qjs(get)]
    fn signal(&self) -> Result<Value<'js>> {
        Ok(self.signal.clone())
    }

    pub(crate) fn error(&mut self, ctx: Ctx<'js>, err: Value<'js>) -> Result<()> {
        WritableStream::error_stream_from(&self.stream, &ctx, err)
    }
}

// =============================================================================
// WritableStreamDefaultWriter
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct WritableStreamDefaultWriter<'js> {
    stream: Option<Class<'js, WritableStream<'js>>>,
    ready_promise: Promise<'js>,
    ready_resolve: Option<Function<'js>>,
    ready_reject: Option<Function<'js>>,
    closed_promise: Promise<'js>,
    closed_resolve: Option<Function<'js>>,
    closed_reject: Option<Function<'js>>,
}

impl<'js> WritableStreamDefaultWriter<'js> {
    pub(crate) fn create(
        ctx: &Ctx<'js>,
        stream: Class<'js, WritableStream<'js>>,
    ) -> Result<Class<'js, Self>> {
        let (ready_promise, ready_resolve, ready_reject) = Promise::new(ctx)?;
        ready_resolve.call::<_, ()>(())?;
        let (closed_promise, closed_resolve, closed_reject) = Promise::new(ctx)?;
        Ok(Class::instance(
            ctx.clone(),
            Self {
                stream: Some(stream),
                ready_promise,
                ready_resolve: Some(ready_resolve),
                ready_reject: Some(ready_reject),
                closed_promise,
                closed_resolve: Some(closed_resolve),
                closed_reject: Some(closed_reject),
            },
        )?)
    }

    pub(crate) fn make_ready_pending(&mut self, ctx: &Ctx<'js>) -> Result<()> {
        if self.ready_resolve.is_some() {
            return Ok(());
        }
        let (promise, resolve, reject) = Promise::new(ctx)?;
        self.ready_promise = promise;
        self.ready_resolve = Some(resolve);
        self.ready_reject = Some(reject);
        Ok(())
    }

    pub(crate) fn resolve_ready(&mut self, ctx: &Ctx<'js>) -> Result<()> {
        if let Some(resolve) = self.ready_resolve.take() {
            resolve_later(ctx, resolve, ())?;
        }
        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> WritableStreamDefaultWriter<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "WritableStreamDefaultWriter cannot be constructed directly",
        ))
    }

    fn write(&mut self, ctx: Ctx<'js>, chunk: Value<'js>) -> Result<Promise<'js>> {
        let stream = match &self.stream {
            Some(s) => s.clone(),
            None => {
                return Err(throw_type_error(&ctx, "This writer has been released"));
            }
        };

        let size = {
            let s = stream.try_borrow()?;
            if s.state == WritableStreamState::Errored {
                let (promise, _, reject) = Promise::new(&ctx)?;
                reject_later(&ctx, reject, s.error.clone().unwrap())?;
                return Ok(promise);
            }
            if s.state != WritableStreamState::Writable {
                return Err(throw_type_error(
                    &ctx,
                    "WritableStream is closed or errored",
                ));
            }
            if let Some(size_fn) = &s.size_fn {
                match size_fn.call::<_, f64>((chunk.clone(),)) {
                    Ok(size) => {
                        if !size.is_finite() || size < 0.0 {
                            let err: Value = ctx.eval(
                                "new TypeError('Chunk size must be a non-negative finite number')",
                            )?;
                            WritableStream::error_stream_from(&stream, &ctx, err.clone())?;
                            return Err(throw_type_error(
                                &ctx,
                                "Chunk size must be a non-negative finite number",
                            ));
                        }
                        size
                    }
                    Err(_e) => {
                        let err: Value = ctx.eval("new TypeError('size algorithm failed')")?;
                        WritableStream::error_stream_from(&stream, &ctx, err.clone())?;
                        return Err(throw_type_error(&ctx, "size algorithm failed"));
                    }
                }
            } else {
                1.0
            }
        };

        let (promise, resolve, reject) = Promise::new(&ctx)?;
        {
            let mut s = stream.try_borrow_mut()?;
            s.queue.push(WriteRecord {
                chunk,
                size,
                resolve: resolve.clone(),
                reject: reject.clone(),
            });
            s.queue_size += size;
        }
        let full = {
            let s = stream.try_borrow()?;
            let in_flight = s.current_write.as_ref().map_or(0.0, |r| r.size);
            s.queue_size + in_flight >= s.high_water_mark
        };
        if full {
            self.make_ready_pending(&ctx)?;
            {
                let mut s = stream.try_borrow_mut()?;
                s.writer_ready_resolve = self.ready_resolve.clone();
                s.writer_ready_reject = self.ready_reject.clone();
            }
        } else {
            self.resolve_ready(&ctx)?;
            {
                let mut s = stream.try_borrow_mut()?;
                s.writer_ready_resolve = None;
                s.writer_ready_reject = None;
            }
        }
        WritableStream::process_queue(&stream, &ctx)?;
        Ok(promise)
    }

    fn close(&mut self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
        let stream = match &self.stream {
            Some(s) => s.clone(),
            None => {
                return Err(throw_type_error(&ctx, "This writer has been released"));
            }
        };
        {
            let mut s = stream.try_borrow_mut()?;
            if s.state == WritableStreamState::Errored {
                let (promise, _, reject) = Promise::new(&ctx)?;
                reject_later(&ctx, reject, s.error.clone().unwrap())?;
                return Ok(promise);
            }
            if s.state != WritableStreamState::Writable {
                return Err(throw_type_error(
                    &ctx,
                    "WritableStream is closed or errored",
                ));
            }
            if s.close_requested {
                return Ok(s.close_promise.clone());
            }
            s.close_requested = true;
        }
        WritableStream::process_queue(&stream, &ctx)?;
        let promise = stream.try_borrow()?.close_promise.clone();
        Ok(promise)
    }

    fn abort(&mut self, ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<Promise<'js>> {
        let stream = self.stream.take();
        if let Some(stream) = stream {
            return WritableStream::abort_internal(&stream, &ctx, reason.0);
        }
        resolved_promise(&ctx)
    }

    fn release_lock(&mut self, _ctx: Ctx<'js>) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut s = stream.try_borrow_mut()?;
            s.writer = None;
            s.writer_ready_resolve = None;
            s.writer_ready_reject = None;
            s.writer_closed_resolve = None;
            s.writer_closed_reject = None;
        }
        Ok(())
    }

    #[qjs(get)]
    fn ready(&self) -> Result<Promise<'js>> {
        Ok(self.ready_promise.clone())
    }

    #[qjs(get)]
    fn closed(&self) -> Result<Promise<'js>> {
        Ok(self.closed_promise.clone())
    }
}

// =============================================================================
// WritableStream
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct WritableStream<'js> {
    controller: Option<Class<'js, WritableStreamDefaultController<'js>>>,
    writer: Option<Class<'js, WritableStreamDefaultWriter<'js>>>,
    writer_ready_resolve: Option<Function<'js>>,
    writer_ready_reject: Option<Function<'js>>,
    writer_closed_resolve: Option<Function<'js>>,
    writer_closed_reject: Option<Function<'js>>,
    state: WritableStreamState,
    error: Option<Value<'js>>,
    queue: Vec<WriteRecord<'js>>,
    queue_size: f64,
    writing: bool,
    current_write: Option<WriteRecord<'js>>,
    close_requested: bool,
    close_promise: Promise<'js>,
    close_resolve: Option<Function<'js>>,
    close_reject: Option<Function<'js>>,
    high_water_mark: f64,
    size_fn: Option<Function<'js>>,
    underlying_sink: Object<'js>,
}

impl<'js> WritableStream<'js> {
    pub(crate) fn resolve_writer_ready_if_needed(
        stream: &Class<'js, Self>,
        ctx: &Ctx<'js>,
    ) -> Result<()> {
        let (ready_resolve, full) = {
            let s = stream.try_borrow()?;
            let in_flight = s.current_write.as_ref().map_or(0.0, |r| r.size);
            let total = s.queue_size + in_flight;
            (s.writer_ready_resolve.clone(), total >= s.high_water_mark)
        };
        if !full {
            if let Some(resolve) = ready_resolve {
                resolve_later(ctx, resolve, ())?;
                stream.try_borrow_mut()?.writer_ready_resolve = None;
            }
        }
        Ok(())
    }

    pub(crate) fn process_queue(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        loop {
            let close_now = {
                let s = stream.try_borrow()?;
                s.state == WritableStreamState::Writable
                    && s.close_requested
                    && !s.writing
                    && s.queue.is_empty()
            };

            if close_now {
                {
                    let mut s = stream.try_borrow_mut()?;
                    s.writing = true;
                }
                let (sink, controller) = {
                    let s = stream.try_borrow()?;
                    (s.underlying_sink.clone(), s.controller.clone().unwrap())
                };
                let close_fn = get_optional_function(&sink, "close")?;
                if let Some(close_fn) = close_fn {
                    let result = close_fn.call((controller,))?;
                    let stream_ref = stream.clone();
                    let stream_ref_err = stream.clone();
                    handle_maybe_promise(
                        ctx,
                        result,
                        move |ctx| WritableStream::finish_close(&stream_ref, &ctx),
                        move |_ctx, err| {
                            WritableStream::error_stream_from(&stream_ref_err, &_ctx, err)
                        },
                    )?;
                } else {
                    WritableStream::finish_close(stream, ctx)?;
                }
                return Ok(());
            }

            let should_write = {
                let s = stream.try_borrow()?;
                s.state == WritableStreamState::Writable && !s.writing && !s.queue.is_empty()
            };
            if !should_write {
                WritableStream::resolve_writer_ready_if_needed(stream, ctx)?;
                return Ok(());
            }

            let record = {
                let mut s = stream.try_borrow_mut()?;
                let record = s.queue.remove(0);
                s.queue_size -= record.size;
                s.writing = true;
                record
            };

            let chunk = record.chunk.clone();
            let (sink, controller) = {
                let s = stream.try_borrow()?;
                (s.underlying_sink.clone(), s.controller.clone().unwrap())
            };
            let write_fn = match get_optional_function(&sink, "write")? {
                Some(f) => f,
                None => {
                    resolve_later(ctx, record.resolve, ())?;
                    {
                        let mut s = stream.try_borrow_mut()?;
                        s.writing = false;
                    }
                    WritableStream::process_queue(stream, ctx)?;
                    return Ok(());
                }
            };

            {
                let mut s = stream.try_borrow_mut()?;
                s.current_write = Some(record);
            }

            let result = write_fn.call((chunk, controller))?;
            let stream_ref = stream.clone();
            let stream_ref_err = stream.clone();
            handle_maybe_promise(
                ctx,
                result,
                move |ctx| WritableStream::finish_write(&stream_ref, &ctx),
                move |_ctx, err| WritableStream::error_stream_from(&stream_ref_err, &_ctx, err),
            )?;
            return Ok(());
        }
    }

    pub(crate) fn finish_write(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        let record = {
            let mut s = stream.try_borrow_mut()?;
            let record = s.current_write.take().unwrap();
            s.writing = false;
            record
        };
        resolve_later(ctx, record.resolve, ())?;
        WritableStream::process_queue(stream, ctx)?;
        WritableStream::resolve_writer_ready_if_needed(stream, ctx)?;
        Ok(())
    }

    pub(crate) fn finish_close(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        let (close_resolve, closed_resolve, ready_resolve) = {
            let mut s = stream.try_borrow_mut()?;
            s.state = WritableStreamState::Closed;
            s.writing = false;
            (
                s.close_resolve.take(),
                s.writer_closed_resolve.take(),
                s.writer_ready_resolve.take(),
            )
        };
        if let Some(resolve) = close_resolve {
            resolve_later(ctx, resolve, ())?;
        }
        if let Some(resolve) = closed_resolve {
            resolve_later(ctx, resolve, ())?;
        }
        if let Some(resolve) = ready_resolve {
            resolve_later(ctx, resolve, ())?;
        }
        Ok(())
    }

    pub(crate) fn error_stream_from(
        stream: &Class<'js, Self>,
        ctx: &Ctx<'js>,
        err: Value<'js>,
    ) -> Result<()> {
        let (
            current,
            queued,
            close_reject,
            ready_reject,
            closed_reject,
            _controller,
            close_requested,
        ) = {
            let mut s = stream.try_borrow_mut()?;
            if s.state != WritableStreamState::Writable {
                return Ok(());
            }
            s.state = WritableStreamState::Errored;
            s.error = Some(err.clone());
            s.writer_ready_resolve = None;
            s.writer_closed_resolve = None;
            (
                s.current_write.take(),
                std::mem::take(&mut s.queue),
                s.close_reject.take(),
                s.writer_ready_reject.take(),
                s.writer_closed_reject.take(),
                s.controller.clone(),
                s.close_requested,
            )
        };
        if let Some(record) = current {
            let _ = reject_later(ctx, record.reject, err.clone());
        }
        for record in queued {
            let _ = reject_later(ctx, record.reject, err.clone());
        }
        if close_requested {
            if let Some(reject) = close_reject {
                let _ = reject_later(ctx, reject, err.clone());
            }
        }
        if let Some(reject) = ready_reject {
            let _ = reject_later(ctx, reject, err.clone());
        }
        if let Some(reject) = closed_reject {
            let _ = reject_later(ctx, reject, err.clone());
        }
        Ok(())
    }

    pub(crate) fn abort_internal(
        stream: &Class<'js, Self>,
        ctx: &Ctx<'js>,
        reason: Option<Value<'js>>,
    ) -> Result<Promise<'js>> {
        let reason = reason.unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        let controller = stream.try_borrow()?.controller.clone();
        if let Some(controller) = controller {
            let _ = controller.try_borrow()?.abort_signal(ctx, reason.clone());
        }
        WritableStream::error_stream_from(stream, ctx, reason.clone())?;
        let sink = stream.try_borrow()?.underlying_sink.clone();
        if let Some(abort) = get_optional_function(&sink, "abort")? {
            let (promise, resolve, reject) = Promise::new(ctx)?;
            let result = abort.call((reason.clone(),))?;
            let resolve2 = resolve.clone();
            handle_maybe_promise(
                ctx,
                result,
                move |_ctx| {
                    resolve_later(&_ctx, resolve2.clone(), ())?;
                    Ok(())
                },
                move |_ctx, err| {
                    reject_later(&_ctx, reject.clone(), err)?;
                    Ok(())
                },
            )?;
            Ok(promise)
        } else {
            resolved_promise(ctx)
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> WritableStream<'js> {
    #[qjs(constructor)]
    fn new(
        ctx: Ctx<'js>,
        underlying_sink: Opt<Object<'js>>,
        strategy: Opt<Object<'js>>,
    ) -> Result<Class<'js, Self>> {
        let sink = underlying_sink
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
        let strategy = strategy
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
        let high_water_mark = strategy.get::<&str, f64>("highWaterMark").unwrap_or(1.0);
        let size_fn = get_optional_function(&strategy, "size")?;

        let (close_promise, close_resolve, close_reject) = Promise::new(&ctx)?;

        let stream = Class::instance(
            ctx.clone(),
            Self {
                controller: None,
                writer: None,
                writer_ready_resolve: None,
                writer_ready_reject: None,
                writer_closed_resolve: None,
                writer_closed_reject: None,
                state: WritableStreamState::Writable,
                error: None,
                queue: Vec::new(),
                queue_size: 0.0,
                writing: false,
                current_write: None,
                close_requested: false,
                close_promise,
                close_resolve: Some(close_resolve),
                close_reject: Some(close_reject),
                high_water_mark,
                size_fn,
                underlying_sink: sink.clone(),
            },
        )?;

        let abort_controller: Object = ctx.eval("new AbortController()")?;
        let signal: Value = abort_controller.get("signal")?;
        let controller = Class::instance(
            ctx.clone(),
            WritableStreamDefaultController::new(stream.clone(), signal, abort_controller),
        )?;
        stream.try_borrow_mut()?.controller = Some(controller.clone());
        stream.set("_controller", controller.clone())?;

        if let Some(start) = get_optional_function(&sink, "start")? {
            let _: std::result::Result<(), rquickjs::Error> = start.call((controller,));
        }

        Ok(stream)
    }

    #[qjs(get)]
    fn locked(&self) -> bool {
        self.writer.is_some()
    }

    fn get_writer(
        ctx: Ctx<'js>,
        this: rquickjs::function::This<Class<'js, Self>>,
    ) -> Result<Class<'js, WritableStreamDefaultWriter<'js>>> {
        let mut stream = this.0.try_borrow_mut()?;
        if stream.writer.is_some() {
            return Err(throw_type_error(
                &ctx,
                "WritableStream already has a writer",
            ));
        }
        let writer = WritableStreamDefaultWriter::create(&ctx, this.0.clone())?;
        let writer_ref = writer.try_borrow()?;
        stream.writer_ready_resolve = writer_ref.ready_resolve.clone();
        stream.writer_ready_reject = writer_ref.ready_reject.clone();
        stream.writer_closed_resolve = writer_ref.closed_resolve.clone();
        stream.writer_closed_reject = writer_ref.closed_reject.clone();
        drop(writer_ref);
        stream.writer = Some(writer.clone());
        Ok(writer)
    }

    fn abort(
        ctx: Ctx<'js>,
        this: rquickjs::function::This<Class<'js, Self>>,
        reason: Opt<Value<'js>>,
    ) -> Result<Promise<'js>> {
        WritableStream::abort_internal(&this.0, &ctx, reason.0)
    }

    fn close(
        ctx: Ctx<'js>,
        this: rquickjs::function::This<Class<'js, Self>>,
    ) -> Result<Promise<'js>> {
        {
            let mut s = this.0.try_borrow_mut()?;
            if s.state != WritableStreamState::Writable {
                return Err(throw_type_error(
                    &ctx,
                    "WritableStream is closed or errored",
                ));
            }
            if s.close_requested {
                return Ok(s.close_promise.clone());
            }
            s.close_requested = true;
        }
        WritableStream::process_queue(&this.0, &ctx)?;
        Ok(this.0.try_borrow()?.close_promise.clone())
    }
}
