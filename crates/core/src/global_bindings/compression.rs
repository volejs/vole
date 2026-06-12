use rquickjs::{Ctx, Function, Result, TypedArray};
use std::io::{Read, Write};

fn compression_encode<'js>(
    ctx: Ctx<'js>,
    format: String,
    data: TypedArray<'js, u8>,
) -> Result<TypedArray<'js, u8>> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let compressed = match format.as_str() {
        "gzip" => {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        "deflate" => {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        "deflate-raw" => {
            let mut encoder =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&bytes)
                .map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    TypedArray::new_copy(ctx, &compressed)
}

fn compression_decode<'js>(
    ctx: Ctx<'js>,
    format: String,
    data: TypedArray<'js, u8>,
) -> Result<TypedArray<'js, u8>> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let decompressed = match format.as_str() {
        "gzip" => {
            let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        "deflate" => {
            let mut decoder = flate2::read::ZlibDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        "deflate-raw" => {
            let mut decoder = flate2::read::DeflateDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    TypedArray::new_copy(ctx, &decompressed)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    // Expose the raw encode/decode helpers temporarily so the JS shim can
    // reference them without importing a module.
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

    // Clean up the temporary helpers now that the closures have captured them.
    let _ = ctx.globals().remove("__compression_encode");
    let _ = ctx.globals().remove("__compression_decode");

    Ok(())
}
