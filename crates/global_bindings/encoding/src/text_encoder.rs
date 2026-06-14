use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Class, Constructor, Ctx, Function, Object, Result, TypedArray, Value,
};

#[derive(Clone, Debug)]
pub struct TextEncoder;

unsafe impl<'js> rquickjs::JsLifetime<'js> for TextEncoder {
    type Changed<'to> = TextEncoder;
}

impl<'js> Trace<'js> for TextEncoder {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for TextEncoder {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

fn text_encoder_encode<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, TextEncoder>>,
    text: String,
) -> Result<TypedArray<'js, u8>> {
    TypedArray::new_copy(ctx, text.into_bytes())
}

fn text_encoder_encode_into<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, TextEncoder>>,
    text: String,
    dest: TypedArray<'js, u8>,
) -> Result<Object<'js>> {
    let bytes = text.into_bytes();
    let dest_len = dest.len();
    let written = bytes.len().min(dest_len);

    if written > 0 {
        let arr = dest.as_object();
        for (i, b) in bytes.iter().take(written).enumerate() {
            arr.set(i as u32, *b)?;
        }
    }

    let result = Object::new(ctx)?;
    result.set("read", bytes.len() as u32)?;
    result.set("written", written as u32)?;
    Ok(result)
}

fn text_encoder_ctor() -> Result<TextEncoder> {
    Ok(TextEncoder)
}

impl<'js> JsClass<'js> for TextEncoder {
    const NAME: &'static str = "TextEncoder";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("encode", Function::new(ctx.clone(), text_encoder_encode)?)?;
        proto.set(
            "encodeInto",
            Function::new(ctx.clone(), text_encoder_encode_into)?,
        )?;
        proto.set("encoding", Function::new(ctx.clone(), || "utf-8"))?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<TextEncoder, _, _>(ctx.clone(), text_encoder_ctor)?;
        Ok(Some(constr))
    }
}
