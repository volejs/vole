use std::io::{self, Write};

use crate::CompressionFormat;

/// 持有一个跨 chunk 复用的压缩器。
///
/// Web `CompressionStream` 的核心语义是“一个流对应一个压缩上下文”。输入可以被
/// 任意切分成多个 chunk，但输出必须等价于对完整字节序列压缩一次。因此这里不能像
/// 简化实现那样对每个 chunk 单独创建 encoder 并 `finish()`，否则 gzip/deflate frame
/// 边界和压缩字典都会错误。
pub(crate) enum StreamingEncoder {
    Gzip(flate2::write::GzEncoder<Vec<u8>>),
    Deflate(flate2::write::ZlibEncoder<Vec<u8>>),
    DeflateRaw(flate2::write::DeflateEncoder<Vec<u8>>),
    Brotli(brotlic::CompressorWriter<Vec<u8>>),
}

impl StreamingEncoder {
    pub(crate) fn new(format: CompressionFormat) -> io::Result<Self> {
        match format {
            CompressionFormat::Gzip => Ok(Self::Gzip(flate2::write::GzEncoder::new(
                Vec::new(),
                flate2::Compression::default(),
            ))),
            CompressionFormat::Deflate => Ok(Self::Deflate(flate2::write::ZlibEncoder::new(
                Vec::new(),
                flate2::Compression::default(),
            ))),
            CompressionFormat::DeflateRaw => Ok(Self::DeflateRaw(
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default()),
            )),
            CompressionFormat::Brotli => {
                Ok(Self::Brotli(brotlic::CompressorWriter::new(Vec::new())))
            }
        }
    }

    pub(crate) fn write_chunk(&mut self, input: &[u8]) -> io::Result<Vec<u8>> {
        match self {
            Self::Gzip(encoder) => {
                encoder.write_all(input)?;
                encoder.flush()?;
                Ok(std::mem::take(encoder.get_mut()))
            }
            Self::Deflate(encoder) => {
                encoder.write_all(input)?;
                encoder.flush()?;
                Ok(std::mem::take(encoder.get_mut()))
            }
            Self::DeflateRaw(encoder) => {
                encoder.write_all(input)?;
                encoder.flush()?;
                Ok(std::mem::take(encoder.get_mut()))
            }
            Self::Brotli(encoder) => {
                encoder.write_all(input)?;
                encoder.flush()?;
                Ok(std::mem::take(encoder.get_mut()))
            }
        }
    }

    pub(crate) fn finish(self) -> io::Result<Vec<u8>> {
        match self {
            Self::Gzip(encoder) => encoder.finish(),
            Self::Deflate(encoder) => encoder.finish(),
            Self::DeflateRaw(encoder) => encoder.finish(),
            Self::Brotli(encoder) => encoder
                .into_inner()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}
