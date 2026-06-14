use rquickjs::{function::This, Ctx, Function, Object, Result, Value};

pub fn init(ctx: &Ctx) -> Result<()> {
    ctx.globals()
        .set("reportError", Function::new(ctx.clone(), report_error_impl))?;
    Ok(())
}

fn report_error_impl<'js>(ctx: Ctx<'js>, error: Value<'js>) {
    if dispatch_error_event(&ctx, &error).unwrap_or(false) {
        return;
    }
    eprintln!("{}", format_error(error));
}

fn dispatch_error_event<'js>(ctx: &Ctx<'js>, error: &Value<'js>) -> Result<bool> {
    let globals = ctx.globals();
    let event_ctor: Function = match globals.get("Event") {
        Ok(event_ctor) => event_ctor,
        Err(_) => return Ok(false),
    };
    let dispatch_event: Function = match globals.get("dispatchEvent") {
        Ok(dispatch_event) => dispatch_event,
        Err(_) => return Ok(false),
    };

    let event: Object = event_ctor.call(("error",))?;
    event.set("error", error.clone())?;
    event.set("message", format_error(error.clone()))?;
    let _ = dispatch_event.call::<_, bool>((This(globals), event))?;
    Ok(true)
}

fn format_error(value: Value<'_>) -> String {
    if value.is_undefined() {
        "undefined".to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if value.is_string() {
        value
            .as_string()
            .and_then(|s| s.to_string().ok())
            .unwrap_or_else(|| "string".to_string())
    } else if let Some(obj) = value.as_object() {
        let mut parts = Vec::new();
        if let Ok(name) = obj.get::<_, String>("name") {
            parts.push(name);
        }
        if let Ok(msg) = obj.get::<_, String>("message") {
            parts.push(msg);
        }
        if parts.is_empty() {
            "[object Error]".to_string()
        } else {
            parts.join(": ")
        }
    } else {
        "[object Error]".to_string()
    }
}
