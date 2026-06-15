use std::io::{self, Write};

use crate::CompressionFormat;

/// 持有一个跨 chunk 复用的解压器。
///
/// 这保留了 LLRT `StreamingDecoder` 的核心能力：压缩数据可以按任意边界进入，解压器
/// 维护内部状态并在后续 chunk 或 `finish()` 时继续输出剩余数据。这个能力后续也能被
/// fetch/http 的 `Content-Encoding` 解码复用。
pub enum StreamingDecoder {
    Gzip(flate2::write::GzDecoder<Vec<u8>>),
    Deflate(flate2::write::ZlibDecoder<Vec<u8>>),
    DeflateRaw(flate2::write::DeflateDecoder<Vec<u8>>),
    Brotli(brotlic::DecompressorWriter<Vec<u8>>),
}

impl StreamingDecoder {
    pub(crate) fn new(format: CompressionFormat) -> io::Result<Self> {
        match format {
            CompressionFormat::Gzip => Ok(Self::Gzip(flate2::write::GzDecoder::new(Vec::new()))),
            CompressionFormat::Deflate => Ok(Self::Deflate(flate2::write::ZlibDecoder::new(
                Vec::new(),
            ))),
            CompressionFormat::DeflateRaw => Ok(Self::DeflateRaw(
                flate2::write::DeflateDecoder::new(Vec::new()),
            )),
            CompressionFormat::Brotli => Ok(Self::Brotli(brotlic::DecompressorWriter::new(Vec::new()))),
        }
    }

    pub fn from_encoding(encoding: &str) -> io::Result<Self> {
        match encoding {
            "gzip" => Self::new(CompressionFormat::Gzip),
            "deflate" => Self::new(CompressionFormat::Deflate),
            "deflate-raw" => Self::new(CompressionFormat::DeflateRaw),
            "br" | "brotli" => Self::new(CompressionFormat::Brotli),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported encoding: {encoding}"),
            )),
        }
    }

    pub fn write_chunk(&mut self, input: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            Self::Gzip(decoder) => {
                decoder.write_all(input)?;
                decoder.flush()?;
                Ok(std::mem::take(decoder.get_mut()))
            },
            Self::Deflate(decoder) => {
                decoder.write_all(input)?;
                decoder.flush()?;
                Ok(std::mem::take(decoder.get_mut()))
            },
            Self::DeflateRaw(decoder) => {
                decoder.write_all(input)?;
                decoder.flush()?;
                Ok(std::mem::take(decoder.get_mut()))
            },
            Self::Brotli(decoder) => {
                decoder.write_all(input)?;
                decoder.flush()?;
                Ok(std::mem::take(decoder.get_mut()))
            },
        }
    }

    pub fn decompress_chunk(&mut self, input: &[u8]) -> io::Result<Vec<u8>> {
        self.write_chunk(input)
    }

    pub fn finish(self) -> io::Result<Vec<u8>> {
        match self {
            Self::Gzip(decoder) => decoder.finish(),
            Self::Deflate(decoder) => decoder.finish(),
            Self::DeflateRaw(decoder) => decoder.finish(),
            Self::Brotli(decoder) => decoder
                .into_inner()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}
