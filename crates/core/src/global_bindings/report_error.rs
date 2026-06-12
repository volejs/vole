use rquickjs::{Ctx, Function, Value};

pub fn init(ctx: &Ctx) -> Result<(), rquickjs::Error> {
    ctx.globals()
        .set("reportError", Function::new(ctx.clone(), report_error_impl))?;
    Ok(())
}

fn report_error_impl(_ctx: Ctx, error: Value) {
    eprintln!("{}", format_error(&error));
}

fn format_error(value: &Value) -> String {
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
