use std::time::{SystemTime, UNIX_EPOCH};

use rquickjs::{
    ArrayBuffer, Coerced, Ctx, Exception, IntoJs, Object, Result, Value, atom::PredefinedAtom,
    class::Trace, function::Opt,
};

use super::blob::Blob;

#[rquickjs::class]
#[derive(Trace, Clone, rquickjs::JsLifetime)]
pub struct File<'js> {
    blob: Blob<'js>,
    filename: String,
    last_modified: i64,
}

#[rquickjs::methods]
impl<'js> File<'js> {
    #[qjs(constructor)]
    fn new(
        ctx: Ctx<'js>,
        data: Value<'js>,
        filename: Coerced<String>,
        options: Opt<Value<'js>>,
    ) -> Result<Self> {
        let mut last_modified = now_millis();

        if let Some(ref opts) = options.0 {
            if opts.is_bool() || opts.is_float() || opts.is_int() || opts.is_string() {
                return Err(Exception::throw_type(&ctx, "Invalid options"));
            }

            if let Some(v) = opts.as_object() {
                if let Some(x) = v.get::<_, Option<Coerced<i64>>>("lastModified")? {
                    last_modified = x.0;
                }
            }
        }

        let blob = Blob::new(ctx, Opt(Some(data)), options)?;

        Ok(Self {
            blob,
            filename: filename.0,
            last_modified,
        })
    }

    #[qjs(get)]
    pub fn size(&self) -> usize {
        self.blob.size()
    }

    #[qjs(get)]
    pub fn name(&self) -> String {
        self.filename.clone()
    }

    #[qjs(get, rename = "type")]
    pub fn mime_type(&self) -> String {
        self.blob.mime_type()
    }

    #[qjs(get, rename = "lastModified")]
    pub fn last_modified(&self) -> i64 {
        self.last_modified
    }

    #[qjs(get, rename = "webkitRelativePath")]
    pub fn webkit_relative_path(&self) -> &'static str {
        ""
    }

    pub fn slice(
        &self,
        ctx: Ctx<'js>,
        start: Opt<isize>,
        end: Opt<isize>,
        content_type: Opt<String>,
    ) -> Result<Blob<'js>> {
        self.blob.slice(ctx, start, end, content_type)
    }

    pub async fn text(&mut self) -> String {
        self.blob.text().await
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer(&self, ctx: Ctx<'js>) -> Result<ArrayBuffer<'js>> {
        self.blob.array_buffer(ctx).await
    }

    pub async fn bytes(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.blob.bytes(ctx).await
    }

    pub fn stream(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.blob.stream(ctx)
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(File)
    }
}

impl<'js> File<'js> {
    pub fn from_bytes(
        ctx: &Ctx<'js>,
        data: Vec<u8>,
        filename: String,
        mime_type: Option<String>,
    ) -> Result<Self> {
        let options = Opt(Some({
            let obj = Object::new(ctx.clone())?;
            obj.set(
                "type",
                mime_type
                    .clone()
                    .unwrap_or("application/octet-stream".into())
                    .into_js(ctx)?,
            )?;
            obj.into_js(ctx)?
        }));
        let blob = Blob::new(ctx.clone(), Opt(Some(data.into_js(ctx)?)), options)?;

        Ok(Self {
            blob,
            filename,
            last_modified: now_millis(),
        })
    }

    pub fn get_blob(&self) -> Blob<'js> {
        self.blob.clone()
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
