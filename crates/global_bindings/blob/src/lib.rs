use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Array, ArrayBuffer, Class, Constructor, Ctx, FromJs, IntoJs, Object, Result, TypedArray, Value,
};

// --- Blob ---

#[derive(Clone, Debug)]
pub struct Blob {
    data: Vec<u8>,
    mime_type: String,
}

impl Blob {
    fn new<'js>(
        ctx: &Ctx<'js>,
        parts: Option<Value<'js>>,
        options: Option<Object<'js>>,
    ) -> Result<Self> {
        let mime_type = options
            .as_ref()
            .and_then(|o| o.get::<&str, String>("type").ok())
            .unwrap_or_default();

        let mut data = Vec::new();

        if let Some(parts) = parts {
            if let Ok(arr) = parts.get::<Array<'js>>() {
                for i in 0..arr.len() {
                    let item = arr.get::<Value<'js>>(i)?;
                    append_value(ctx, &mut data, item)?;
                }
            } else {
                append_value(ctx, &mut data, parts)?;
            }
        }

        Ok(Self { data, mime_type })
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn text(&self) -> Result<String> {
        String::from_utf8(self.data.clone()).map_err(|_| rquickjs::Error::Exception)
    }

    fn array_buffer<'js>(&self, ctx: Ctx<'js>) -> Result<ArrayBuffer<'js>> {
        ArrayBuffer::new_copy(ctx, &self.data)
    }

    fn slice(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        content_type: Option<String>,
    ) -> Result<Blob> {
        let len = self.data.len() as i64;
        let start = start.unwrap_or(0).clamp(0, len) as usize;
        let end = end.unwrap_or(len).clamp(0, len) as usize;
        let start = start.min(end);
        let end = end.max(start);
        Ok(Blob {
            data: self.data[start..end].to_vec(),
            mime_type: content_type.unwrap_or_default(),
        })
    }
}

fn append_value<'js>(_ctx: &Ctx<'js>, data: &mut Vec<u8>, value: Value<'js>) -> Result<()> {
    if value.is_string() {
        let s = value.as_string().ok_or(rquickjs::Error::Exception)?;
        data.extend_from_slice(s.to_string()?.as_bytes());
    } else if let Ok(ab) = value.get::<ArrayBuffer<'js>>() {
        data.extend_from_slice(<ArrayBuffer<'js> as AsRef<[u8]>>::as_ref(&ab));
    } else if let Ok(ta) = value.get::<TypedArray<'js, u8>>() {
        data.extend_from_slice(<TypedArray<'js, u8> as AsRef<[u8]>>::as_ref(&ta));
    } else {
        return Err(rquickjs::Error::Exception);
    }
    Ok(())
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for Blob {
    type Changed<'to> = Blob;
}

impl<'js> Trace<'js> for Blob {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for Blob {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for Blob {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Class::<Blob>::from_js(ctx, value)?.try_borrow()?.clone())
    }
}

fn blob_size<'js>(this: This<Class<'js, Blob>>) -> Result<usize> {
    Ok(this.try_borrow()?.size())
}

fn blob_type<'js>(this: This<Class<'js, Blob>>) -> Result<String> {
    Ok(this.try_borrow()?.mime_type.clone())
}

fn blob_text<'js>(this: This<Class<'js, Blob>>) -> Result<String> {
    this.try_borrow()?.text()
}

fn blob_array_buffer<'js>(ctx: Ctx<'js>, this: This<Class<'js, Blob>>) -> Result<ArrayBuffer<'js>> {
    this.try_borrow()?.array_buffer(ctx)
}

fn blob_bytes<'js>(ctx: Ctx<'js>, this: This<Class<'js, Blob>>) -> Result<TypedArray<'js, u8>> {
    let buf = this.try_borrow()?.array_buffer(ctx.clone())?;
    TypedArray::new_copy(ctx, <ArrayBuffer<'js> as AsRef<[u8]>>::as_ref(&buf))
}

fn blob_slice<'js>(
    this: This<Class<'js, Blob>>,
    start: Option<i64>,
    end: Option<i64>,
    content_type: Option<String>,
) -> Result<Blob> {
    this.try_borrow()?.slice(start, end, content_type)
}

fn blob_constructor<'js>(
    ctx: Ctx<'js>,
    parts: rquickjs::function::Opt<Value<'js>>,
    options: rquickjs::function::Opt<Object<'js>>,
) -> Result<Blob> {
    Blob::new(&ctx, parts.0, options.0)
}

impl<'js> JsClass<'js> for Blob {
    const NAME: &'static str = "Blob";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("size", rquickjs::Function::new(ctx.clone(), blob_size)?)?;
        proto.set("type", rquickjs::Function::new(ctx.clone(), blob_type)?)?;
        proto.set("text", rquickjs::Function::new(ctx.clone(), blob_text)?)?;
        proto.set(
            "arrayBuffer",
            rquickjs::Function::new(ctx.clone(), blob_array_buffer)?,
        )?;
        proto.set("bytes", rquickjs::Function::new(ctx.clone(), blob_bytes)?)?;
        proto.set("slice", rquickjs::Function::new(ctx.clone(), blob_slice)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<Blob, _, _>(ctx.clone(), blob_constructor)?;
        Ok(Some(constr))
    }
}

// --- File ---

#[derive(Clone, Debug)]
pub struct File {
    blob: Blob,
    name: String,
    last_modified: i64,
}

impl File {
    fn new<'js>(
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
    Ok(this.try_borrow()?.blob.mime_type.clone())
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
    parts: rquickjs::function::Opt<Value<'js>>,
    name: String,
    options: rquickjs::function::Opt<Object<'js>>,
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

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<Blob>::define(&ctx.globals())?;
    Class::<File>::define(&ctx.globals())?;

    // Convert Blob prototype methods to getters
    ctx.eval::<(), _>(
        r#"
        (function() {
            const wrapGetter = (proto, name) => {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: function() { return fn.call(this); },
                    enumerable: true,
                    configurable: true
                });
            };
            wrapGetter(Blob.prototype, 'size');
            wrapGetter(Blob.prototype, 'type');
            wrapGetter(File.prototype, 'name');
            wrapGetter(File.prototype, 'lastModified');
            wrapGetter(File.prototype, 'size');
            wrapGetter(File.prototype, 'type');
            Object.defineProperty(File.prototype, 'webkitRelativePath', {
                get: function() { return ''; },
                enumerable: true,
                configurable: true
            });

            Blob.prototype.stream = function() {
                var self = this;
                // ReadableStream is defined later in the init sequence; deferring
                // the lookup until .stream() is called avoids ordering issues.
                return new ReadableStream({
                    start: function(controller) {
                        var buf = self.arrayBuffer();
                        controller.enqueue(new Uint8Array(buf));
                        controller.close();
                    }
                });
            };

        })();
    "#,
    )?;

    Ok(())
}
