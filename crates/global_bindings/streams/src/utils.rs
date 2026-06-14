use rquickjs::{Ctx, Error, Exception, Function, IntoJs, Object, Promise, Result, Value};

pub fn get_optional_function<'js>(obj: &Object<'js>, name: &str) -> Result<Option<Function<'js>>> {
    if !obj.contains_key(name)? {
        return Ok(None);
    }
    let value: Value = obj.get(name)?;
    if let Ok(f) = value.get::<Function<'js>>() {
        Ok(Some(f))
    } else {
        Ok(None)
    }
}

pub fn throw_type_error(ctx: &Ctx, message: &str) -> Error {
    Exception::throw_type(ctx, message)
}

pub fn handle_maybe_promise<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
    on_fulfilled: impl Fn(Ctx<'js>) -> Result<()> + 'js,
    on_rejected: impl Fn(Ctx<'js>, Value<'js>) -> Result<()> + 'js,
) -> Result<()> {
    if let Some(promise) = value.as_promise() {
        let fulfill = Function::new(ctx.clone(), move |ctx: Ctx<'js>| on_fulfilled(ctx))?;
        let reject = Function::new(ctx.clone(), move |ctx: Ctx<'js>, err: Value<'js>| {
            on_rejected(ctx, err)
        })?;
        let helper: Function = ctx.globals().get("__vole_promise_then")?;
        helper.call::<_, ()>((promise.clone(), fulfill, reject))?;
        Ok(())
    } else {
        on_fulfilled(ctx.clone())
    }
}

pub fn resolved_promise<'js>(ctx: &Ctx<'js>) -> Result<Promise<'js>> {
    let (promise, resolve, _) = Promise::new(ctx)?;
    resolve.call::<_, ()>(())?;
    Ok(promise)
}

pub fn resolve_later<'js>(
    ctx: &Ctx<'js>,
    resolve: Function<'js>,
    value: impl IntoJs<'js>,
) -> Result<()> {
    let helper: Function = ctx.globals().get("__vole_resolve_later")?;
    helper.call::<_, ()>((resolve, value))?;
    Ok(())
}

pub fn reject_later<'js>(ctx: &Ctx<'js>, reject: Function<'js>, reason: Value<'js>) -> Result<()> {
    let helper: Function = ctx.globals().get("__vole_reject_later")?;
    helper.call::<_, ()>((reject, reason))?;
    Ok(())
}

pub fn schedule_task<'js>(ctx: &Ctx<'js>, callback: Function<'js>) -> Result<()> {
    let helper: Function = ctx.globals().get("__vole_schedule_task")?;
    helper.call::<_, ()>((callback,))?;
    Ok(())
}
