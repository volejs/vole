use rquickjs::{Ctx, Function, Result, Value};

fn structured_clone_impl<'js>(ctx: Ctx<'js>, value: Value<'js>) -> Result<Value<'js>> {
    // Use serde_json::Value as intermediate for deep copy
    let json_val: serde_json::Value =
        rquickjs_serde::from_value(value).map_err(|_| rquickjs::Error::Exception)?;
    rquickjs_serde::to_value(ctx, json_val).map_err(|_| rquickjs::Error::Exception)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    ctx.globals().set(
        "structuredClone",
        Function::new(ctx.clone(), structured_clone_impl)?,
    )?;
    Ok(())
}
