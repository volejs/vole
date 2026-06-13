use rquickjs::{Ctx, Result, TypedArray};
use std::io::Write;

pub fn compression_encode<'js>(
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
