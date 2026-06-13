use crate::blob::Blob;
use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    ArrayBuffer, Class, Constructor, Ctx, FromJs, IntoJs, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct File {
    blob: Blob,
    name: String,
    last_modified: i64,
}

impl File {
    pub fn new<'js>(
        ctx: &Ctx<'js>,
        parts: Option<Value<'js>>,
        name: String,
        options: Option<Object<'js>>,
    ) -> Result<Self> {
        let blob = Blob::new(ctx, parts, options.clone())?;
        let last_modified = options
            .as_ref()
            .and_then(|o| o.get::<&str, i64>("lastModified").ok())
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64
            });
        Ok(Self {
            blob,
            name,
            last_modified,
        })
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for File {
    type Changed<'to> = File;
}

impl<'js> Trace<'js> for File {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for File {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for File {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Class::<File>::from_js(ctx, value)?.try_borrow()?.clone())
    }
}

fn file_name<'js>(this: This<Class<'js, File>>) -> Result<String> {
    Ok(this.try_borrow()?.name.clone())
}

fn file_last_modified<'js>(this: This<Class<'js, File>>) -> Result<i64> {
    Ok(this.try_borrow()?.last_modified)
}

fn file_size<'js>(this: This<Class<'js, File>>) -> Result<usize> {
    Ok(this.try_borrow()?.blob.size())
}

fn file_type<'js>(this: This<Class<'js, File>>) -> Result<String> {
    Ok(this.try_borrow()?.blob.mime_type().to_string())
}

fn file_text<'js>(this: This<Class<'js, File>>) -> Result<String> {
    this.try_borrow()?.blob.text()
}

fn file_array_buffer<'js>(ctx: Ctx<'js>, this: This<Class<'js, File>>) -> Result<ArrayBuffer<'js>> {
    this.try_borrow()?.blob.array_buffer(ctx)
}

fn file_slice<'js>(
    this: This<Class<'js, File>>,
    start: Option<i64>,
    end: Option<i64>,
    content_type: Option<String>,
) -> Result<Blob> {
    this.try_borrow()?.blob.slice(start, end, content_type)
}

fn file_constructor<'js>(
    ctx: Ctx<'js>,
    parts: Opt<Value<'js>>,
    name: String,
    options: Opt<Object<'js>>,
) -> Result<File> {
    File::new(&ctx, parts.0, name, options.0)
}

impl<'js> JsClass<'js> for File {
    const NAME: &'static str = "File";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("name", rquickjs::Function::new(ctx.clone(), file_name)?)?;
        proto.set(
            "lastModified",
            rquickjs::Function::new(ctx.clone(), file_last_modified)?,
        )?;
        proto.set("size", rquickjs::Function::new(ctx.clone(), file_size)?)?;
        proto.set("type", rquickjs::Function::new(ctx.clone(), file_type)?)?;
        proto.set("text", rquickjs::Function::new(ctx.clone(), file_text)?)?;
        proto.set(
            "arrayBuffer",
            rquickjs::Function::new(ctx.clone(), file_array_buffer)?,
        )?;
        proto.set("slice", rquickjs::Function::new(ctx.clone(), file_slice)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<File, _, _>(ctx.clone(), file_constructor)?;
        Ok(Some(constr))
    }
}
