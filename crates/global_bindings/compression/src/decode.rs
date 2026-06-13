use rquickjs::{Ctx, Result, TypedArray};
use std::io::Read;

pub fn compression_decode<'js>(
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
