use rquickjs::{Ctx, Function, Result};

pub mod decode;
pub mod encode;

pub use decode::compression_decode;
pub use encode::compression_encode;

pub fn init(ctx: &Ctx) -> Result<()> {
    ctx.globals().set(
        "__compression_encode",
        Function::new(ctx.clone(), compression_encode),
    )?;
    ctx.globals().set(
        "__compression_decode",
        Function::new(ctx.clone(), compression_decode),
    )?;

    ctx.eval::<(), _>(
        r#"
(function(encode, decode) {
    class CompressionStream {
        constructor(format = "deflate") {
            this._format = format;
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    const encoded = encode(self._format, chunk);
                    controller.enqueue(encoded);
                },
            });
        }
        get readable() {
            return this._transform.readable;
        }
        get writable() {
            return this._transform.writable;
        }
    }

    class DecompressionStream {
        constructor(format = "deflate") {
            this._format = format;
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    const decoded = decode(self._format, chunk);
                    controller.enqueue(decoded);
                },
            });
        }
        get readable() {
            return this._transform.readable;
        }
        get writable() {
            return this._transform.writable;
        }
    }

    globalThis.CompressionStream = CompressionStream;
    globalThis.DecompressionStream = DecompressionStream;
})(__compression_encode, __compression_decode);
"#,
    )?;

    let _ = ctx.globals().remove("__compression_encode");
    let _ = ctx.globals().remove("__compression_decode");

    Ok(())
}
