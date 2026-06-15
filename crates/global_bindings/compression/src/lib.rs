use std::io;

use rquickjs::{
    Class, Coerced, Constructor, Ctx, Exception, Function, JsLifetime, Object, Result, TypedArray,
    Value,
    atom::PredefinedAtom,
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
};
use vole_core::bytes::ObjectBytes;

mod decode;
mod encode;

pub use decode::StreamingDecoder;
use encode::StreamingEncoder;

#[derive(Clone, Copy)]
pub(crate) enum CompressionFormat {
    Gzip,
    Deflate,
    DeflateRaw,
    Brotli,
}

impl CompressionFormat {
    fn from_web_format<'js>(ctx: &Ctx<'js>, format: &str) -> Result<Self> {
        match format {
            "gzip" => Ok(Self::Gzip),
            "deflate" => Ok(Self::Deflate),
            "deflate-raw" => Ok(Self::DeflateRaw),
            "brotli" => Ok(Self::Brotli),
            _ => Err(Exception::throw_type(
                ctx,
                "Compression format must be 'gzip', 'deflate', 'deflate-raw', or 'brotli'",
            )),
        }
    }
}

struct CompressionEncoderHandle {
    inner: Option<StreamingEncoder>,
}

unsafe impl<'js> JsLifetime<'js> for CompressionEncoderHandle {
    type Changed<'to> = CompressionEncoderHandle;
}

impl<'js> Trace<'js> for CompressionEncoderHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> JsClass<'js> for CompressionEncoderHandle {
    const NAME: &'static str = "CompressionEncoderHandle";
    type Mutable = Writable;

    fn prototype(_ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        Ok(None)
    }

    fn constructor(_ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(None)
    }
}

struct CompressionDecoderHandle {
    inner: Option<StreamingDecoder>,
}

unsafe impl<'js> JsLifetime<'js> for CompressionDecoderHandle {
    type Changed<'to> = CompressionDecoderHandle;
}

impl<'js> Trace<'js> for CompressionDecoderHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> JsClass<'js> for CompressionDecoderHandle {
    const NAME: &'static str = "CompressionDecoderHandle";
    type Mutable = Writable;

    fn prototype(_ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        Ok(None)
    }

    fn constructor(_ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(None)
    }
}

#[rquickjs::class]
#[derive(JsLifetime, Trace, Clone)]
pub struct CompressionStream<'js> {
    transform: Object<'js>,
}

#[rquickjs::methods]
impl<'js> CompressionStream<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, format: Opt<Coerced<String>>) -> Result<Self> {
        let format = parse_required_format(&ctx, format, "CompressionStream")?;
        let handle = Class::instance(
            ctx.clone(),
            CompressionEncoderHandle {
                inner: Some(StreamingEncoder::new(format).map_err(|err| io_exception(&ctx, err))?),
            },
        )?;
        let transformer = encoder_transformer(ctx.clone(), handle)?;
        let transform = construct_transform_stream(ctx, transformer)?;
        Ok(Self { transform })
    }

    #[qjs(get)]
    pub fn readable(&self) -> Result<Value<'js>> {
        self.transform.get("readable")
    }

    #[qjs(get)]
    pub fn writable(&self) -> Result<Value<'js>> {
        self.transform.get("writable")
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(CompressionStream)
    }
}

#[rquickjs::class]
#[derive(JsLifetime, Trace, Clone)]
pub struct DecompressionStream<'js> {
    transform: Object<'js>,
}

#[rquickjs::methods]
impl<'js> DecompressionStream<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, format: Opt<Coerced<String>>) -> Result<Self> {
        let format = parse_required_format(&ctx, format, "DecompressionStream")?;
        let handle = Class::instance(
            ctx.clone(),
            CompressionDecoderHandle {
                inner: Some(StreamingDecoder::new(format).map_err(|err| io_exception(&ctx, err))?),
            },
        )?;
        let transformer = decoder_transformer(ctx.clone(), handle)?;
        let transform = construct_transform_stream(ctx, transformer)?;
        Ok(Self { transform })
    }

    #[qjs(get)]
    pub fn readable(&self) -> Result<Value<'js>> {
        self.transform.get("readable")
    }

    #[qjs(get)]
    pub fn writable(&self) -> Result<Value<'js>> {
        self.transform.get("writable")
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(DecompressionStream)
    }
}

fn parse_required_format<'js>(
    ctx: &Ctx<'js>,
    format: Opt<Coerced<String>>,
    constructor_name: &str,
) -> Result<CompressionFormat> {
    let format = format.0.ok_or_else(|| {
        Exception::throw_type(
            ctx,
            &format!("{} requires a compression format", constructor_name),
        )
    })?;
    CompressionFormat::from_web_format(ctx, &format.to_string())
}

fn construct_transform_stream<'js>(ctx: Ctx<'js>, transformer: Object<'js>) -> Result<Object<'js>> {
    let ctor: Constructor<'js> = ctx.globals().get("TransformStream")?;
    ctor.construct((transformer,))
}

fn encoder_transformer<'js>(
    ctx: Ctx<'js>,
    handle: Class<'js, CompressionEncoderHandle>,
) -> Result<Object<'js>> {
    let transformer = Object::new(ctx.clone())?;
    transformer.set("_handle", handle)?;
    transformer.set("transform", Function::new(ctx.clone(), encoder_transform)?)?;
    transformer.set("flush", Function::new(ctx, encoder_flush)?)?;
    Ok(transformer)
}

fn decoder_transformer<'js>(
    ctx: Ctx<'js>,
    handle: Class<'js, CompressionDecoderHandle>,
) -> Result<Object<'js>> {
    let transformer = Object::new(ctx.clone())?;
    transformer.set("_handle", handle)?;
    transformer.set("transform", Function::new(ctx.clone(), decoder_transform)?)?;
    transformer.set("flush", Function::new(ctx, decoder_flush)?)?;
    Ok(transformer)
}

fn io_exception<'js>(ctx: &Ctx<'js>, err: io::Error) -> rquickjs::Error {
    Exception::throw_message(ctx, &err.to_string())
}

fn bytes_from_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Vec<u8>> {
    let bytes = ObjectBytes::from(ctx, value)?;
    Ok(bytes.as_bytes(ctx)?.as_ref().to_vec())
}

fn typed_array_from_vec<'js>(ctx: Ctx<'js>, bytes: Vec<u8>) -> Result<TypedArray<'js, u8>> {
    TypedArray::new_copy(ctx, bytes.as_slice())
}

fn enqueue_output<'js>(ctx: Ctx<'js>, controller: Object<'js>, output: Vec<u8>) -> Result<()> {
    if output.is_empty() {
        return Ok(());
    }
    let bytes = typed_array_from_vec(ctx, output)?;
    let enqueue: Function<'js> = controller.get("enqueue")?;
    enqueue.call::<_, ()>((This(controller), bytes))
}

fn encoder_write<'js>(
    ctx: &Ctx<'js>,
    handle: Class<'js, CompressionEncoderHandle>,
    chunk: Value<'js>,
) -> Result<Vec<u8>> {
    let bytes = bytes_from_value(ctx, &chunk)?;
    let mut handle = handle.try_borrow_mut()?;
    let encoder = handle
        .inner
        .as_mut()
        .ok_or_else(|| Exception::throw_type(ctx, "CompressionStream is already closed"))?;
    encoder
        .write_chunk(bytes.as_slice())
        .map_err(|err| io_exception(ctx, err))
}

fn encoder_finish<'js>(
    ctx: &Ctx<'js>,
    handle: Class<'js, CompressionEncoderHandle>,
) -> Result<Vec<u8>> {
    let mut handle = handle.try_borrow_mut()?;
    let encoder = handle
        .inner
        .take()
        .ok_or_else(|| Exception::throw_type(ctx, "CompressionStream is already closed"))?;
    encoder.finish().map_err(|err| io_exception(ctx, err))
}

fn decoder_write<'js>(
    ctx: &Ctx<'js>,
    handle: Class<'js, CompressionDecoderHandle>,
    chunk: Value<'js>,
) -> Result<Vec<u8>> {
    let bytes = bytes_from_value(ctx, &chunk)?;
    let mut handle = handle.try_borrow_mut()?;
    let decoder = handle
        .inner
        .as_mut()
        .ok_or_else(|| Exception::throw_type(ctx, "DecompressionStream is already closed"))?;
    decoder
        .write_chunk(bytes.as_slice())
        .map_err(|err| io_exception(ctx, err))
}

fn decoder_finish<'js>(
    ctx: &Ctx<'js>,
    handle: Class<'js, CompressionDecoderHandle>,
) -> Result<Vec<u8>> {
    let mut handle = handle.try_borrow_mut()?;
    let decoder = handle
        .inner
        .take()
        .ok_or_else(|| Exception::throw_type(ctx, "DecompressionStream is already closed"))?;
    decoder.finish().map_err(|err| io_exception(ctx, err))
}

fn encoder_transform<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    chunk: Value<'js>,
    controller: Object<'js>,
) -> Result<()> {
    let handle: Class<'js, CompressionEncoderHandle> = this.0.get("_handle")?;
    let output = encoder_write(&ctx, handle, chunk)?;
    enqueue_output(ctx, controller, output)
}

fn encoder_flush<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    controller: Object<'js>,
) -> Result<()> {
    let handle: Class<'js, CompressionEncoderHandle> = this.0.get("_handle")?;
    let output = encoder_finish(&ctx, handle)?;
    enqueue_output(ctx, controller, output)
}

fn decoder_transform<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    chunk: Value<'js>,
    controller: Object<'js>,
) -> Result<()> {
    let handle: Class<'js, CompressionDecoderHandle> = this.0.get("_handle")?;
    let output = decoder_write(&ctx, handle, chunk)?;
    enqueue_output(ctx, controller, output)
}

fn decoder_flush<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    controller: Object<'js>,
) -> Result<()> {
    let handle: Class<'js, CompressionDecoderHandle> = this.0.get("_handle")?;
    let output = decoder_finish(&ctx, handle)?;
    enqueue_output(ctx, controller, output)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    Class::<CompressionStream>::define(&globals)?;
    Class::<DecompressionStream>::define(&globals)?;

    Ok(())
}
