use std::cell::{Cell, RefCell};

use rquickjs::{Coerced, Ctx, Object, Result, Value, atom::PredefinedAtom, function::Opt};
use vole_core::{bytes::ObjectBytes, encoding::Encoder, object::ObjectExt, result::ResultExt};

#[rquickjs::class]
#[derive(rquickjs::class::Trace, rquickjs::JsLifetime)]
pub struct TextDecoder {
    #[qjs(skip_trace)]
    encoder: Encoder,
    fatal: bool,
    ignore_bom: bool,
    #[qjs(skip_trace)]
    pending: RefCell<Vec<u8>>,
    #[qjs(skip_trace)]
    bom_seen: Cell<bool>,
}

fn utf8_incomplete_tail(bytes: &[u8]) -> usize {
    let len = bytes.len();
    for i in 1..=4.min(len) {
        let b = bytes[len - i];
        if b < 0x80 {
            return 0;
        }
        if b >= 0xC0 {
            let expected = match b {
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => return 0,
            };
            if i >= expected {
                return 0;
            }
            let tail = &bytes[len - i + 1..];
            for (j, &c) in tail.iter().enumerate() {
                if j == 0 {
                    let valid = match b {
                        0xE0 => (0xA0..=0xBF).contains(&c),
                        0xED => (0x80..=0x9F).contains(&c),
                        0xF0 => (0x90..=0xBF).contains(&c),
                        0xF4 => (0x80..=0x8F).contains(&c),
                        _ => (0x80..=0xBF).contains(&c),
                    };
                    if !valid {
                        return 0;
                    }
                } else if c & 0xC0 != 0x80 {
                    return 0;
                }
            }
            return i;
        }
    }
    0
}

#[rquickjs::methods]
impl<'js> TextDecoder {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        label: Opt<Coerced<String>>,
        options: Opt<Object<'js>>,
    ) -> Result<Self> {
        let mut fatal = false;
        let mut ignore_bom = false;

        let label = label.0.map(|label| label.0);
        let encoder = Encoder::from_optional_str(label.as_deref())
            .or_throw_range(&ctx, "The encoding label is invalid")?;

        if let Some(opts) = options.0 {
            if let Some(opt) = opts.get_optional("fatal")? {
                fatal = opt;
            }
            if let Some(opt) = opts.get_optional("ignoreBOM")? {
                ignore_bom = opt;
            }
        }

        Ok(Self {
            encoder,
            fatal,
            ignore_bom,
            pending: RefCell::new(Vec::new()),
            bom_seen: Cell::new(false),
        })
    }

    #[qjs(get)]
    fn encoding(&self) -> &str {
        self.encoder.as_label()
    }

    #[qjs(get)]
    fn fatal(&self) -> bool {
        self.fatal
    }

    #[qjs(get, rename = "ignoreBOM")]
    fn ignore_bom(&self) -> bool {
        self.ignore_bom
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(TextDecoder)
    }

    pub fn decode(
        &self,
        ctx: Ctx<'js>,
        bytes: Opt<ObjectBytes<'js>>,
        options: Opt<Value<'js>>,
    ) -> Result<String> {
        let mut stream = false;
        if let Some(opts) = options.0.as_ref().and_then(|v| v.as_object()) {
            if let Some(s) = opts.get_optional("stream")? {
                stream = s;
            }
        }

        let input_bytes: &[u8] = bytes
            .0
            .as_ref()
            .and_then(ObjectBytes::as_bytes_opt)
            .unwrap_or(&[]);

        let mut pending = self.pending.borrow_mut();

        let combined: Vec<u8>;
        let mut data: &[u8] = if pending.is_empty() {
            input_bytes
        } else {
            pending.extend_from_slice(input_bytes);
            combined = std::mem::take(&mut *pending);
            &combined
        };

        if !stream {
            self.bom_seen.set(false);
        }

        if !self.ignore_bom && !self.bom_seen.get() {
            let skip = match self.encoder {
                Encoder::Utf8 if data.starts_with(&[0xEF, 0xBB, 0xBF]) => 3,
                Encoder::Utf16le if data.starts_with(&[0xFF, 0xFE]) => 2,
                Encoder::Utf16be if data.starts_with(&[0xFE, 0xFF]) => 2,
                _ => 0,
            };

            if skip > 0 {
                self.bom_seen.set(true);
                data = &data[skip..];
            } else if stream
                && match self.encoder {
                    Encoder::Utf8 => data == [0xEF] || data == [0xEF, 0xBB],
                    Encoder::Utf16le => data == [0xFF],
                    Encoder::Utf16be => data == [0xFE],
                    _ => false,
                }
            {
                *pending = data.to_vec();
                return Ok(String::new());
            } else if !data.is_empty() {
                self.bom_seen.set(true);
            }
        }

        let mut decode_end = data.len();

        if stream {
            match self.encoder {
                Encoder::Utf8 => {
                    decode_end -= utf8_incomplete_tail(data);
                }
                Encoder::Utf16le | Encoder::Utf16be => {
                    let odd = data.len() % 2;
                    decode_end -= odd;
                    if decode_end >= 2 {
                        let last_u16 = if matches!(self.encoder, Encoder::Utf16le) {
                            u16::from_le_bytes([data[decode_end - 2], data[decode_end - 1]])
                        } else {
                            u16::from_be_bytes([data[decode_end - 2], data[decode_end - 1]])
                        };
                        if (0xD800..=0xDBFF).contains(&last_u16) {
                            decode_end -= 2;
                        }
                    }
                }
                _ => {}
            }

            if decode_end < data.len() {
                *pending = data[decode_end..].to_vec();
            }
        }

        self.encoder
            .encode_to_string(&data[..decode_end], !self.fatal)
            .or_throw_type(&ctx, "The encoded input cannot be decoded")
    }
}
