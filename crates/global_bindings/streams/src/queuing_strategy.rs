use rquickjs::{Ctx, Result};

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct ByteLengthQueuingStrategy<'js> {
    #[qjs(get)]
    high_water_mark: f64,
    #[qjs(skip_trace)]
    _phantom: std::marker::PhantomData<&'js ()>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> ByteLengthQueuingStrategy<'js> {
    #[qjs(constructor)]
    fn new(
        _ctx: Ctx<'js>,
        options: rquickjs::function::Opt<rquickjs::Object<'js>>,
    ) -> Result<Self> {
        let hwm = options
            .0
            .as_ref()
            .and_then(|o| o.get::<&str, f64>("highWaterMark").ok())
            .unwrap_or(0.0);
        Ok(Self {
            high_water_mark: hwm,
            _phantom: std::marker::PhantomData,
        })
    }

    fn size(&self, chunk: rquickjs::Value<'js>) -> Result<f64> {
        if let Ok(ab) = chunk.get::<rquickjs::ArrayBuffer<'js>>() {
            Ok(ab.len() as f64)
        } else if let Ok(ta) = chunk.get::<rquickjs::TypedArray<'js, u8>>() {
            Ok(ta.len() as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[derive(rquickjs::JsLifetime, rquickjs::class::Trace)]
#[rquickjs::class]
pub struct CountQueuingStrategy<'js> {
    #[qjs(get)]
    high_water_mark: f64,
    #[qjs(skip_trace)]
    _phantom: std::marker::PhantomData<&'js ()>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> CountQueuingStrategy<'js> {
    #[qjs(constructor)]
    fn new(
        _ctx: Ctx<'js>,
        options: rquickjs::function::Opt<rquickjs::Object<'js>>,
    ) -> Result<Self> {
        let hwm = options
            .0
            .as_ref()
            .and_then(|o| o.get::<&str, f64>("highWaterMark").ok())
            .unwrap_or(1.0);
        Ok(Self {
            high_water_mark: hwm,
            _phantom: std::marker::PhantomData,
        })
    }

    fn size(&self) -> f64 {
        1.0
    }
}
