use rquickjs::{
    class::Trace,
    function::{Opt, This},
    Class, Ctx, Error, Function, Object, Promise, Result, Value,
};

use crate::utils::{
    get_optional_function, handle_maybe_promise, reject_later, resolve_later, resolved_promise,
    schedule_task, throw_type_error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadableStreamState {
    Readable,
    Closed,
    Errored,
}

impl<'js> Trace<'js> for ReadableStreamState {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[derive(rquickjs::JsLifetime, Trace)]
struct PendingRead<'js> {
    resolve: Function<'js>,
    reject: Function<'js>,
}

#[derive(rquickjs::JsLifetime, Trace)]
struct PendingByobRead<'js> {
    view: Object<'js>,
    resolve: Function<'js>,
    reject: Function<'js>,
}

// =============================================================================
// ReadableStreamDefaultController
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ReadableStreamDefaultController<'js> {
    stream: Class<'js, ReadableStream<'js>>,
    high_water_mark: f64,
    size_fn: Option<Function<'js>>,
}

impl<'js> ReadableStreamDefaultController<'js> {
    pub(crate) fn new(
        stream: Class<'js, ReadableStream<'js>>,
        high_water_mark: f64,
        size_fn: Option<Function<'js>>,
    ) -> Self {
        Self {
            stream,
            high_water_mark,
            size_fn,
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ReadableStreamDefaultController<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "ReadableStreamDefaultController cannot be constructed directly",
        ))
    }

    pub(crate) fn enqueue(&mut self, ctx: Ctx<'js>, chunk: Value<'js>) -> Result<()> {
        let size = if let Some(size_fn) = &self.size_fn {
            match size_fn.call::<_, f64>((chunk.clone(),)) {
                Ok(s) => s,
                Err(_e) => {
                    let err: Value = ctx.eval("new TypeError('size algorithm failed')")?;
                    return self.error(ctx, err);
                }
            }
        } else {
            1.0
        };
        if !size.is_finite() || size < 0.0 {
            let err: Value =
                ctx.eval("new TypeError('Chunk size must be a non-negative finite number')")?;
            return self.error(ctx, err);
        }
        let mut stream = self.stream.try_borrow_mut()?;
        stream.enqueue_value(&ctx, chunk, size)?;
        Ok(())
    }

    pub(crate) fn close(&mut self, ctx: Ctx<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        stream.close_stream(&ctx)?;
        Ok(())
    }

    pub(crate) fn error(&mut self, ctx: Ctx<'js>, err: Value<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        stream.error_stream(&ctx, err)?;
        Ok(())
    }

    #[qjs(get)]
    pub(crate) fn desired_size(&self) -> Result<f64> {
        let stream = self.stream.try_borrow()?;
        Ok(self.high_water_mark - stream.queue_total_size)
    }
}

// =============================================================================
// ReadableByteStreamController
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ReadableByteStreamController<'js> {
    stream: Class<'js, ReadableStream<'js>>,
    high_water_mark: f64,
}

impl<'js> ReadableByteStreamController<'js> {
    pub(crate) fn new(stream: Class<'js, ReadableStream<'js>>, high_water_mark: f64) -> Self {
        Self {
            stream,
            high_water_mark,
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ReadableByteStreamController<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "ReadableByteStreamController cannot be constructed directly",
        ))
    }

    pub(crate) fn enqueue(&mut self, ctx: Ctx<'js>, chunk: Value<'js>) -> Result<()> {
        let len: usize = {
            let helper: Function = ctx.globals().get("__vole_byte_length")?;
            helper.call((chunk.clone(),))?
        };
        let mut stream = self.stream.try_borrow_mut()?;
        stream.enqueue_value(&ctx, chunk, len as f64)?;
        Ok(())
    }

    pub(crate) fn close(&mut self, ctx: Ctx<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        stream.close_stream(&ctx)?;
        Ok(())
    }

    pub(crate) fn error(&mut self, ctx: Ctx<'js>, err: Value<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        stream.error_stream(&ctx, err)?;
        Ok(())
    }

    #[qjs(get)]
    fn desired_size(&self) -> Result<f64> {
        let stream = self.stream.try_borrow()?;
        Ok(self.high_water_mark - stream.queue_total_size)
    }

    pub(crate) fn respond(&mut self, ctx: Ctx<'js>, bytes_written: Value<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        if stream.state != ReadableStreamState::Readable {
            return Err(throw_type_error(&ctx, "Stream is not readable"));
        }
        let pending = stream
            .pending_byob_read
            .take()
            .ok_or_else(|| throw_type_error(&ctx, "No BYOB read request is pending"))?;
        let total_len: usize = pending.view.get("byteLength")?;
        let bw = bytes_written
            .as_number()
            .ok_or_else(|| throw_type_error(&ctx, "bytesWritten must be a number"))?
            as usize;
        if bw > total_len {
            return Err(throw_type_error(
                &ctx,
                "bytesWritten is greater than the view byte length",
            ));
        }
        drop(stream);
        let new_view: Object = {
            let helper: Function = ctx.globals().get("__vole_byob_respond_view")?;
            helper.call((pending.view.clone(), bw))?
        };
        let obj = Object::new(ctx.clone())?;
        obj.set("done", false)?;
        obj.set("value", new_view)?;
        resolve_later(&ctx, pending.resolve, obj)?;
        ReadableStream::schedule_pull(&self.stream, &ctx)?;
        Ok(())
    }

    pub(crate) fn respond_with_new_view(&mut self, ctx: Ctx<'js>, view: Object<'js>) -> Result<()> {
        let mut stream = self.stream.try_borrow_mut()?;
        if stream.state != ReadableStreamState::Readable {
            return Err(throw_type_error(&ctx, "Stream is not readable"));
        }
        let pending = stream
            .pending_byob_read
            .take()
            .ok_or_else(|| throw_type_error(&ctx, "No BYOB read request is pending"))?;
        let same: bool = {
            let helper: Function = ctx.globals().get("__vole_byob_same_buffer")?;
            helper.call((pending.view.clone(), view.clone()))?
        };
        if !same {
            return Err(throw_type_error(
                &ctx,
                "New view must have the same ArrayBuffer as the original",
            ));
        }
        drop(stream);
        let obj = Object::new(ctx.clone())?;
        obj.set("done", false)?;
        obj.set("value", view)?;
        resolve_later(&ctx, pending.resolve, obj)?;
        ReadableStream::schedule_pull(&self.stream, &ctx)?;
        Ok(())
    }
}

// =============================================================================
// ReadableStreamDefaultReader
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ReadableStreamDefaultReader<'js> {
    stream: Option<Class<'js, ReadableStream<'js>>>,
    closed_promise: Promise<'js>,
    closed_resolve: Option<Function<'js>>,
    closed_reject: Function<'js>,
}

impl<'js> ReadableStreamDefaultReader<'js> {
    pub(crate) fn create(
        ctx: &Ctx<'js>,
        stream: Class<'js, ReadableStream<'js>>,
    ) -> Result<Class<'js, Self>> {
        let (promise, resolve, reject) = Promise::new(ctx)?;
        Ok(Class::instance(
            ctx.clone(),
            Self {
                stream: Some(stream),
                closed_promise: promise,
                closed_resolve: Some(resolve),
                closed_reject: reject,
            },
        )?)
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ReadableStreamDefaultReader<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "ReadableStreamDefaultReader cannot be constructed directly",
        ))
    }

    fn read(&mut self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
        let stream = match &self.stream {
            Some(s) => s.clone(),
            None => {
                return Err(throw_type_error(&ctx, "This reader has been released"));
            }
        };

        {
            let mut s = stream.try_borrow_mut()?;
            if let Some(error) = &s.error {
                let (promise, _, reject) = Promise::new(&ctx)?;
                reject_later(&ctx, reject, error.clone())?;
                return Ok(promise);
            }
            if s.pending_read.is_some() {
                return Err(throw_type_error(&ctx, "A read is already pending"));
            }
            if !s.queue.is_empty() {
                let chunk = s.queue.remove(0);
                let size = 1.0; // size was already accounted in queue_total_size
                s.queue_total_size = (s.queue_total_size - size).max(0.0);
                drop(s);
                ReadableStream::schedule_pull(&stream, &ctx)?;
                let (promise, resolve, _) = Promise::new(&ctx)?;
                let obj = Object::new(ctx.clone())?;
                obj.set("done", false)?;
                obj.set("value", chunk)?;
                resolve_later(&ctx, resolve, obj)?;
                return Ok(promise);
            }
            if s.state == ReadableStreamState::Closed {
                let (promise, resolve, _) = Promise::new(&ctx)?;
                let obj = Object::new(ctx.clone())?;
                obj.set("done", true)?;
                obj.set("value", Value::new_undefined(ctx.clone()))?;
                resolve_later(&ctx, resolve, obj)?;
                return Ok(promise);
            }
        }

        let (promise, resolve, reject) = Promise::new(&ctx)?;
        {
            let mut s = stream.try_borrow_mut()?;
            s.pending_read = Some(PendingRead {
                resolve: resolve.clone(),
                reject: reject.clone(),
            });
        }
        ReadableStream::schedule_pull(&stream, &ctx)?;
        Ok(promise)
    }

    fn release_lock(&mut self, ctx: Ctx<'js>) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut s = stream.try_borrow_mut()?;
            s.reader = None;
            s.reader_closed_resolve.take();
            if let Some(pending) = s.pending_read.take() {
                let err: Value = ctx.eval("new TypeError('Reader released')")?;
                reject_later(&ctx, pending.reject, err)?;
            }
        }
        Ok(())
    }

    #[qjs(get)]
    fn closed(&self) -> Result<Promise<'js>> {
        Ok(self.closed_promise.clone())
    }

    fn cancel(&mut self, ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<Promise<'js>> {
        let stream = self.stream.take();
        if let Some(stream) = stream {
            ReadableStream::release_reader(&stream, &ctx)?;
            let reason = reason
                .0
                .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
            return ReadableStream::cancel_internal(&stream, &ctx, reason);
        }
        resolved_promise(&ctx)
    }
}

// =============================================================================
// ReadableStreamBYOBReader
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ReadableStreamBYOBReader<'js> {
    stream: Option<Class<'js, ReadableStream<'js>>>,
    closed_promise: Promise<'js>,
    closed_resolve: Option<Function<'js>>,
    closed_reject: Function<'js>,
}

impl<'js> ReadableStreamBYOBReader<'js> {
    pub(crate) fn create(
        ctx: &Ctx<'js>,
        stream: Class<'js, ReadableStream<'js>>,
    ) -> Result<Class<'js, Self>> {
        let (promise, resolve, reject) = Promise::new(ctx)?;
        Ok(Class::instance(
            ctx.clone(),
            Self {
                stream: Some(stream),
                closed_promise: promise,
                closed_resolve: Some(resolve),
                closed_reject: reject,
            },
        )?)
    }

    fn resolve_result(ctx: &Ctx<'js>, view: Object<'js>, done: bool) -> Result<Promise<'js>> {
        let (promise, resolve, _) = Promise::new(ctx)?;
        let obj = Object::new(ctx.clone())?;
        obj.set("done", done)?;
        obj.set("value", view)?;
        resolve_later(ctx, resolve, obj)?;
        Ok(promise)
    }

    fn copy_into_view(
        ctx: &Ctx<'js>,
        chunk: Value<'js>,
        view: Object<'js>,
    ) -> Result<(usize, Option<Value<'js>>)> {
        let helper: Function = ctx.globals().get("__vole_byob_copy")?;
        let result: Object = helper.call((chunk, view))?;
        let consumed: usize = result.get("consumed")?;
        let remainder: Value = result.get("remainder")?;
        let remainder = if remainder.is_undefined() {
            None
        } else {
            Some(remainder)
        };
        Ok((consumed, remainder))
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ReadableStreamBYOBReader<'js> {
    #[qjs(constructor)]
    fn constructor(ctx: Ctx<'js>) -> Result<Self> {
        Err(throw_type_error(
            &ctx,
            "ReadableStreamBYOBReader cannot be constructed directly",
        ))
    }

    fn read(&mut self, ctx: Ctx<'js>, view: Object<'js>) -> Result<Promise<'js>> {
        let stream = match &self.stream {
            Some(s) => s.clone(),
            None => {
                return Err(throw_type_error(&ctx, "This reader has been released"));
            }
        };

        {
            let mut s = stream.try_borrow_mut()?;
            if let Some(error) = &s.error {
                let (promise, _, reject) = Promise::new(&ctx)?;
                reject_later(&ctx, reject, error.clone())?;
                return Ok(promise);
            }
            if s.pending_byob_read.is_some() {
                return Err(throw_type_error(&ctx, "A read is already pending"));
            }
            if !s.queue.is_empty() {
                let chunk = s.queue.remove(0);
                let (consumed, remainder) = Self::copy_into_view(&ctx, chunk, view.clone())?;
                if let Some(rem) = remainder {
                    s.queue.insert(0, rem);
                }
                s.queue_total_size = (s.queue_total_size - consumed as f64).max(0.0);
                drop(s);
                ReadableStream::schedule_pull(&stream, &ctx)?;
                return Self::resolve_result(&ctx, view, false);
            }
            if s.state == ReadableStreamState::Closed {
                return Self::resolve_result(&ctx, view, true);
            }
        }

        let (promise, resolve, reject) = Promise::new(&ctx)?;
        {
            let mut s = stream.try_borrow_mut()?;
            s.pending_byob_read = Some(PendingByobRead {
                view,
                resolve: resolve.clone(),
                reject: reject.clone(),
            });
        }
        ReadableStream::schedule_pull(&stream, &ctx)?;
        Ok(promise)
    }

    fn release_lock(&mut self, ctx: Ctx<'js>) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut s = stream.try_borrow_mut()?;
            s.byob_reader = None;
            s.reader_closed_resolve.take();
            if let Some(pending) = s.pending_byob_read.take() {
                let err: Value = ctx.eval("new TypeError('Reader released')")?;
                reject_later(&ctx, pending.reject, err)?;
            }
        }
        Ok(())
    }

    #[qjs(get)]
    fn closed(&self) -> Result<Promise<'js>> {
        Ok(self.closed_promise.clone())
    }

    fn cancel(&mut self, ctx: Ctx<'js>, reason: Opt<Value<'js>>) -> Result<Promise<'js>> {
        let stream = self.stream.take();
        if let Some(stream) = stream {
            ReadableStream::release_byob_reader(&stream, &ctx)?;
            let reason = reason
                .0
                .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
            return ReadableStream::cancel_internal(&stream, &ctx, reason);
        }
        resolved_promise(&ctx)
    }
}

// =============================================================================
// ReadableStream
// =============================================================================

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ReadableStream<'js> {
    controller: Option<Class<'js, ReadableStreamDefaultController<'js>>>,
    byte_controller: Option<Class<'js, ReadableByteStreamController<'js>>>,
    reader: Option<Class<'js, ReadableStreamDefaultReader<'js>>>,
    byob_reader: Option<Class<'js, ReadableStreamBYOBReader<'js>>>,
    reader_closed_resolve: Option<Function<'js>>,
    state: ReadableStreamState,
    error: Option<Value<'js>>,
    queue: Vec<Value<'js>>,
    queue_total_size: f64,
    pending_read: Option<PendingRead<'js>>,
    pending_byob_read: Option<PendingByobRead<'js>>,
    pulling: bool,
    pull_again: bool,
    high_water_mark: f64,
    is_byte_stream: bool,
    underlying_source: Object<'js>,
}

impl<'js> ReadableStream<'js> {
    pub(crate) fn enqueue_value(
        &mut self,
        ctx: &Ctx<'js>,
        value: Value<'js>,
        size: f64,
    ) -> Result<()> {
        if self.state != ReadableStreamState::Readable {
            return Ok(());
        }
        if let Some(pending) = self.pending_read.take() {
            let obj = Object::new(ctx.clone())?;
            obj.set("done", false)?;
            obj.set("value", value)?;
            resolve_later(ctx, pending.resolve.clone(), obj)?;
        } else if let Some(pending) = self.pending_byob_read.take() {
            let (consumed, remainder) =
                ReadableStreamBYOBReader::copy_into_view(ctx, value, pending.view.clone())?;
            let obj = Object::new(ctx.clone())?;
            obj.set("done", false)?;
            obj.set("value", pending.view)?;
            resolve_later(ctx, pending.resolve, obj)?;
            if let Some(rem) = remainder {
                self.queue.push(rem);
                self.queue_total_size += size - consumed as f64;
            }
        } else {
            self.queue.push(value);
            self.queue_total_size += size;
        }
        self.pull_again = true;
        Ok(())
    }

    pub(crate) fn close_stream(&mut self, ctx: &Ctx<'js>) -> Result<()> {
        if self.state != ReadableStreamState::Readable {
            return Ok(());
        }
        self.state = ReadableStreamState::Closed;
        if let Some(pending) = self.pending_read.take() {
            let obj = Object::new(ctx.clone())?;
            obj.set("done", true)?;
            obj.set("value", Value::new_undefined(ctx.clone()))?;
            resolve_later(ctx, pending.resolve.clone(), obj)?;
        }
        if let Some(pending) = self.pending_byob_read.take() {
            let obj = Object::new(ctx.clone())?;
            obj.set("done", true)?;
            obj.set("value", pending.view)?;
            resolve_later(ctx, pending.resolve, obj)?;
        }
        let closed_resolve = self.reader_closed_resolve.take();
        self.reader = None;
        self.byob_reader = None;
        if let Some(resolve) = closed_resolve {
            resolve_later(ctx, resolve, ())?;
        }
        Ok(())
    }

    pub(crate) fn close_stream_silent(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        stream.try_borrow_mut()?.close_stream(ctx)
    }

    pub(crate) fn cancel_internal(
        stream: &Class<'js, Self>,
        ctx: &Ctx<'js>,
        reason: Value<'js>,
    ) -> Result<Promise<'js>> {
        Self::release_reader(stream, ctx)?;
        Self::release_byob_reader(stream, ctx)?;
        let source = {
            let s = stream.try_borrow()?;
            s.underlying_source.clone()
        };
        Self::close_stream_silent(stream, ctx)?;
        if let Some(cancel) = get_optional_function(&source, "cancel")? {
            let (promise, resolve, reject) = Promise::new(ctx)?;
            let result = cancel.call((reason.clone(),))?;
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

    pub(crate) fn error_stream(&mut self, ctx: &Ctx<'js>, err: Value<'js>) -> Result<()> {
        if self.state != ReadableStreamState::Readable {
            return Ok(());
        }
        self.state = ReadableStreamState::Errored;
        self.error = Some(err.clone());
        if let Some(pending) = self.pending_read.take() {
            reject_later(ctx, pending.reject.clone(), err.clone())?;
        }
        if let Some(pending) = self.pending_byob_read.take() {
            reject_later(ctx, pending.reject.clone(), err.clone())?;
        }
        let closed_resolve = self.reader_closed_resolve.take();
        self.reader = None;
        self.byob_reader = None;
        if let Some(resolve) = closed_resolve {
            resolve_later(ctx, resolve, ())?;
        }
        Ok(())
    }

    pub(crate) fn release_reader(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        let mut s = stream.try_borrow_mut()?;
        if s.reader.take().is_some() {
            s.reader_closed_resolve.take();
            if let Some(pending) = s.pending_read.take() {
                let err: Value = ctx.eval("new TypeError('ReadableStream reader released')")?;
                reject_later(ctx, pending.reject, err)?;
            }
        }
        Ok(())
    }

    pub(crate) fn release_byob_reader(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        let mut s = stream.try_borrow_mut()?;
        if s.byob_reader.take().is_some() {
            s.reader_closed_resolve.take();
            if let Some(pending) = s.pending_byob_read.take() {
                let err: Value = ctx.eval("new TypeError('ReadableStream reader released')")?;
                reject_later(ctx, pending.reject, err)?;
            }
        }
        Ok(())
    }

    pub(crate) fn schedule_pull(stream: &Class<'js, Self>, ctx: &Ctx<'js>) -> Result<()> {
        let (pull, controller_value, is_byte) = {
            let mut s = stream.try_borrow_mut()?;
            if s.state != ReadableStreamState::Readable || s.pulling {
                return Ok(());
            }
            let desired_size = s.high_water_mark - s.queue_total_size;
            if s.pending_read.is_none() && s.pending_byob_read.is_none() && desired_size <= 0.0 {
                return Ok(());
            }
            let pull = match get_optional_function(&s.underlying_source, "pull")? {
                Some(f) => f,
                None => return Ok(()),
            };
            s.pulling = true;
            if s.is_byte_stream {
                let c = s.byte_controller.clone().ok_or(Error::InvalidClass {
                    class: "ReadableStream",
                    message: "missing byte controller".to_string(),
                })?;
                (pull, c.as_inner().clone().into_value(), true)
            } else {
                let c = s.controller.clone().ok_or(Error::InvalidClass {
                    class: "ReadableStream",
                    message: "missing controller".to_string(),
                })?;
                (pull, c.as_inner().clone().into_value(), false)
            }
        };

        let result: Value = pull.call((controller_value,))?;
        let stream_ref = stream.clone();
        let stream_err = stream.clone();
        handle_maybe_promise(
            ctx,
            result,
            move |ctx| {
                let pull_again = {
                    let mut s = stream_ref.try_borrow_mut()?;
                    s.pulling = false;
                    std::mem::take(&mut s.pull_again)
                };
                if pull_again {
                    let cb = Function::new(ctx.clone(), {
                        let stream_ref = stream_ref.clone();
                        move |ctx: Ctx<'js>| {
                            ReadableStream::schedule_pull(&stream_ref, &ctx)?;
                            Ok::<(), rquickjs::Error>(())
                        }
                    })?;
                    schedule_task(&ctx, cb)?;
                }
                Ok(())
            },
            move |_ctx, err| {
                if is_byte {
                    let c = stream_err.try_borrow()?.byte_controller.clone().ok_or(
                        Error::InvalidClass {
                            class: "ReadableStream",
                            message: "missing byte controller".to_string(),
                        },
                    )?;
                    c.try_borrow_mut()?.error(_ctx, err)?;
                } else {
                    let c =
                        stream_err
                            .try_borrow()?
                            .controller
                            .clone()
                            .ok_or(Error::InvalidClass {
                                class: "ReadableStream",
                                message: "missing controller".to_string(),
                            })?;
                    c.try_borrow_mut()?.error(_ctx, err)?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ReadableStream<'js> {
    #[qjs(constructor)]
    fn new(
        ctx: Ctx<'js>,
        underlying_source: Opt<Object<'js>>,
        strategy: Opt<Object<'js>>,
    ) -> Result<Class<'js, Self>> {
        let source = underlying_source
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
        let strategy = strategy
            .0
            .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());

        let is_byte_stream = source
            .get::<&str, String>("type")
            .map(|t| t == "bytes")
            .unwrap_or(false);

        let high_water_mark = {
            let has = strategy.get::<&str, Value>("highWaterMark").ok();
            if let Some(v) = has {
                if v.is_undefined() {
                    if is_byte_stream {
                        0.0
                    } else {
                        1.0
                    }
                } else {
                    v.as_number()
                        .unwrap_or(if is_byte_stream { 0.0 } else { 1.0 })
                }
            } else if is_byte_stream {
                0.0
            } else {
                1.0
            }
        };

        let size_fn = if is_byte_stream {
            None
        } else {
            get_optional_function(&strategy, "size")?
        };

        let stream = Class::instance(
            ctx.clone(),
            Self {
                controller: None,
                byte_controller: None,
                reader: None,
                byob_reader: None,
                reader_closed_resolve: None,
                state: ReadableStreamState::Readable,
                error: None,
                queue: Vec::new(),
                queue_total_size: 0.0,
                pending_read: None,
                pending_byob_read: None,
                pulling: false,
                pull_again: false,
                high_water_mark,
                is_byte_stream,
                underlying_source: source.clone(),
            },
        )?;

        if is_byte_stream {
            let controller = Class::instance(
                ctx.clone(),
                ReadableByteStreamController::new(stream.clone(), high_water_mark),
            )?;
            stream.try_borrow_mut()?.byte_controller = Some(controller.clone());
            stream.set("_controller", controller.clone())?;

            if let Some(start) = get_optional_function(&source, "start")? {
                let result = start.call((controller.clone(),))?;
                let stream_ref = stream.clone();
                let controller_ref = controller.clone();
                handle_maybe_promise(
                    &ctx,
                    result,
                    move |ctx| {
                        let pull_again = {
                            let mut s = stream_ref.try_borrow_mut()?;
                            s.pulling = false;
                            std::mem::take(&mut s.pull_again)
                        };
                        if pull_again {
                            let cb = Function::new(ctx.clone(), {
                                let stream_ref = stream_ref.clone();
                                move |ctx: Ctx<'js>| {
                                    ReadableStream::schedule_pull(&stream_ref, &ctx)?;
                                    Ok::<(), rquickjs::Error>(())
                                }
                            })?;
                            schedule_task(&ctx, cb)?;
                        }
                        Ok(())
                    },
                    move |_ctx, err| {
                        controller_ref.try_borrow_mut()?.error(_ctx, err)?;
                        Ok(())
                    },
                )?;
            } else {
                ReadableStream::schedule_pull(&stream, &ctx)?;
            }
        } else {
            let controller = Class::instance(
                ctx.clone(),
                ReadableStreamDefaultController::new(stream.clone(), high_water_mark, size_fn),
            )?;
            stream.try_borrow_mut()?.controller = Some(controller.clone());
            stream.set("_controller", controller.clone())?;

            if let Some(start) = get_optional_function(&source, "start")? {
                let result = start.call((controller.clone(),))?;
                let stream_ref = stream.clone();
                let controller_ref = controller.clone();
                handle_maybe_promise(
                    &ctx,
                    result,
                    move |ctx| {
                        let pull_again = {
                            let mut s = stream_ref.try_borrow_mut()?;
                            s.pulling = false;
                            std::mem::take(&mut s.pull_again)
                        };
                        if pull_again {
                            let cb = Function::new(ctx.clone(), {
                                let stream_ref = stream_ref.clone();
                                move |ctx: Ctx<'js>| {
                                    ReadableStream::schedule_pull(&stream_ref, &ctx)?;
                                    Ok::<(), rquickjs::Error>(())
                                }
                            })?;
                            schedule_task(&ctx, cb)?;
                        }
                        Ok(())
                    },
                    move |_ctx, err| {
                        controller_ref.try_borrow_mut()?.error(_ctx, err)?;
                        Ok(())
                    },
                )?;
            } else {
                ReadableStream::schedule_pull(&stream, &ctx)?;
            }
        }

        Ok(stream)
    }

    #[qjs(get)]
    fn locked(&self) -> bool {
        self.reader.is_some() || self.byob_reader.is_some()
    }

    fn get_reader(
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        options: Opt<Object<'js>>,
    ) -> Result<Object<'js>> {
        let mode: Option<String> = options
            .0
            .as_ref()
            .and_then(|o| o.get::<&str, String>("mode").ok());

        if mode.as_deref() == Some("byob") {
            {
                let s = this.0.try_borrow()?;
                if !s.is_byte_stream {
                    return Err(throw_type_error(
                        &ctx,
                        "Cannot use a BYOB reader on a non-byte stream",
                    ));
                }
                if s.reader.is_some() || s.byob_reader.is_some() {
                    return Err(throw_type_error(
                        &ctx,
                        "ReadableStream already has a reader",
                    ));
                }
            }
            let reader = ReadableStreamBYOBReader::create(&ctx, this.0.clone())?;
            {
                let mut s = this.0.try_borrow_mut()?;
                if s.state != ReadableStreamState::Readable {
                    if let Some(resolve) = reader.try_borrow_mut()?.closed_resolve.take() {
                        resolve_later(&ctx, resolve, ())?;
                    }
                }
                s.reader_closed_resolve = reader.try_borrow()?.closed_resolve.clone();
                s.byob_reader = Some(reader.clone());
            }
            Ok(reader.into_inner())
        } else {
            {
                let s = this.0.try_borrow()?;
                if s.reader.is_some() || s.byob_reader.is_some() {
                    return Err(throw_type_error(
                        &ctx,
                        "ReadableStream already has a reader",
                    ));
                }
            }
            let reader = ReadableStreamDefaultReader::create(&ctx, this.0.clone())?;
            {
                let mut s = this.0.try_borrow_mut()?;
                if s.state != ReadableStreamState::Readable {
                    if let Some(resolve) = reader.try_borrow_mut()?.closed_resolve.take() {
                        resolve_later(&ctx, resolve, ())?;
                    }
                }
                s.reader_closed_resolve = reader.try_borrow()?.closed_resolve.clone();
                s.reader = Some(reader.clone());
            }
            Ok(reader.into_inner())
        }
    }

    fn cancel(
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        reason: Opt<Value<'js>>,
    ) -> Result<Promise<'js>> {
        let reason = reason
            .0
            .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        Self::cancel_internal(&this.0, &ctx, reason)
    }
}
