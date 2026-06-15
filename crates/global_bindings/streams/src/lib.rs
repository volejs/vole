mod queuing_strategy;
pub mod readable;
mod readable_writable_pair;
mod transform;
pub mod utils;
mod writable;

use queuing_strategy::{ByteLengthQueuingStrategy, CountQueuingStrategy};
use readable::{
    ArrayConstructorPrimordials, IteratorPrimordials, ReadableByteStreamController,
    ReadableStreamBYOBReader, ReadableStreamBYOBRequest, ReadableStreamDefaultController,
    ReadableStreamDefaultReader,
};
use rquickjs::{Class, Ctx, Result};
use transform::{TransformStream, TransformStreamDefaultController};
use vole_core::primordials::{BasePrimordials, Primordial};
use vole_core::promise::PromisePrimordials;
use writable::{
    WritableStream, WritableStreamDefaultController, WritableStreamDefaultControllerPrimordials,
    WritableStreamDefaultWriter,
};

pub use readable::stream::{
    ReadableStream, lock_readable_stream, tee_readable_stream, try_sync_drain_closed_stream,
};
pub use readable::{
    CancelAlgorithm, NativePull, NativePullFn, NativePullResult, PullAlgorithm,
    ReadableByteStreamControllerClass, ReadableStreamControllerClass,
    ReadableStreamDefaultControllerClass, StartAlgorithm,
    readable_byte_stream_controller_close_stream, readable_byte_stream_controller_enqueue_bytes,
    readable_byte_stream_controller_enqueue_bytes_borrowed,
    readable_stream_default_controller_close_stream,
    readable_stream_default_controller_enqueue_value,
    readable_stream_default_controller_error_stream,
};

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    BasePrimordials::init(ctx)?;
    PromisePrimordials::init(ctx)?;
    ArrayConstructorPrimordials::init(ctx)?;
    WritableStreamDefaultControllerPrimordials::init(ctx)?;
    IteratorPrimordials::init(ctx)?;

    Class::<ByteLengthQueuingStrategy>::define(&globals)?;
    Class::<CountQueuingStrategy>::define(&globals)?;

    Class::<ReadableByteStreamController>::define(&globals)?;
    Class::<ReadableStream>::define(&globals)?;
    Class::<ReadableStreamBYOBReader>::define(&globals)?;
    Class::<ReadableStreamBYOBRequest>::define(&globals)?;
    Class::<ReadableStreamDefaultController>::define(&globals)?;
    Class::<ReadableStreamDefaultReader>::define(&globals)?;

    Class::<WritableStream>::define(&globals)?;
    Class::<WritableStreamDefaultController>::define(&globals)?;
    Class::<WritableStreamDefaultWriter>::define(&globals)?;

    Class::<TransformStream>::define(&globals)?;
    Class::<TransformStreamDefaultController>::define(&globals)?;

    init_encoding_streams(ctx)?;

    Ok(())
}

fn init_encoding_streams(ctx: &Ctx) -> Result<()> {
    ctx.eval::<(), _>(
        r#"
(function() {
    class TextEncoderStream {
        constructor() {
            this._encoder = new TextEncoder();
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    controller.enqueue(self._encoder.encode(chunk));
                }
            });
        }
        get readable() { return this._transform.readable; }
        get writable() { return this._transform.writable; }
        get encoding() { return "utf-8"; }
    }

    class TextDecoderStream {
        constructor(label = "utf-8", options = {}) {
            this._decoder = new TextDecoder(label, options);
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    const decoded = self._decoder.decode(chunk, { stream: true });
                    if (decoded) controller.enqueue(decoded);
                },
                flush(controller) {
                    const decoded = self._decoder.decode(new Uint8Array(0), { stream: false });
                    if (decoded) controller.enqueue(decoded);
                }
            });
        }
        get readable() { return this._transform.readable; }
        get writable() { return this._transform.writable; }
        get encoding() { return this._decoder.encoding; }
        get fatal() { return this._decoder.fatal; }
        get ignoreBOM() { return this._decoder.ignoreBOM; }
    }

    globalThis.TextEncoderStream = TextEncoderStream;
    globalThis.TextDecoderStream = TextDecoderStream;
})();
"#,
    )
}
