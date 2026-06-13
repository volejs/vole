use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Array, ArrayBuffer, Class, Constructor, Ctx, Function, Object, Result, TypedArray, Value,
};

#[derive(Clone, Debug)]
pub struct TextDecoder {
    #[allow(dead_code)]
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for TextDecoder {
    type Changed<'to> = TextDecoder;
}

impl<'js> Trace<'js> for TextDecoder {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for TextDecoder {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

fn text_decoder_decode<'js>(
    _ctx: Ctx<'js>,
    this: This<Class<'js, TextDecoder>>,
    input: Opt<Value<'js>>,
    _options: Opt<Object<'js>>,
) -> Result<String> {
    let inner = this.try_borrow()?;
    let fatal = inner.fatal;
    let _ignore_bom = inner.ignore_bom;

    let bytes = match input.0 {
        Some(v) if v.is_object() => {
            if let Ok(ta) = v.get::<TypedArray<'js, u8>>() {
                <TypedArray<'js, u8> as AsRef<[u8]>>::as_ref(&ta).to_vec()
            } else if let Ok(ab) = v.get::<ArrayBuffer<'js>>() {
                <ArrayBuffer<'js> as AsRef<[u8]>>::as_ref(&ab).to_vec()
            } else if v.as_object().map_or(false, |o| {
                o.get::<&str, Value>("byteLength").is_ok() && o.get::<&str, Value>("byteOffset").is_ok()
            }) {
                // DataView shim: convert via JS
                return Err(rquickjs::Error::Exception);
            } else {
                if let Ok(arr) = v.get::<Array<'js>>() {
                    let mut bytes = Vec::new();
                    for i in 0..arr.len() {
                        if let Ok(b) = arr.get::<u8>(i) {
                            bytes.push(b);
                        }
                    }
                    bytes
                } else {
                    return Err(rquickjs::Error::Exception);
                }
            }
        }
        Some(_) => return Err(rquickjs::Error::Exception),
        None => Vec::new(),
    };

    if fatal {
        String::from_utf8(bytes).map_err(|_| rquickjs::Error::Exception)
    } else {
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

fn text_decoder_encoding<'js>(this: This<Class<'js, TextDecoder>>) -> Result<String> {
    Ok(this.try_borrow()?.encoding.clone())
}

fn text_decoder_fatal<'js>(this: This<Class<'js, TextDecoder>>) -> Result<bool> {
    Ok(this.try_borrow()?.fatal)
}

fn text_decoder_ignore_bom<'js>(this: This<Class<'js, TextDecoder>>) -> Result<bool> {
    Ok(this.try_borrow()?.ignore_bom)
}

fn text_decoder_ctor(encoding: Opt<String>, options: Opt<Object>) -> Result<TextDecoder> {
    let encoding = encoding.0.unwrap_or_else(|| "utf-8".to_string());
    let fatal = options
        .0
        .as_ref()
        .and_then(|o| o.get::<&str, bool>("fatal").ok())
        .unwrap_or(false);
    let ignore_bom = options
        .0
        .as_ref()
        .and_then(|o| o.get::<&str, bool>("ignoreBOM").ok())
        .unwrap_or(false);
    Ok(TextDecoder {
        encoding,
        fatal,
        ignore_bom,
    })
}

impl<'js> JsClass<'js> for TextDecoder {
    const NAME: &'static str = "TextDecoder";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("decode", Function::new(ctx.clone(), text_decoder_decode)?)?;
        proto.set("encoding", Function::new(ctx.clone(), text_decoder_encoding)?)?;
        proto.set("fatal", Function::new(ctx.clone(), text_decoder_fatal)?)?;
        proto.set("ignoreBOM", Function::new(ctx.clone(), text_decoder_ignore_bom)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<TextDecoder, _, _>(ctx.clone(), text_decoder_ctor)?;
        Ok(Some(constr))
    }
}
