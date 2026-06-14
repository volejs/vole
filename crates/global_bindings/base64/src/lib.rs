use base64::{engine::general_purpose, Engine as _};
use rquickjs::{Ctx, Function, Result, Value};
use vole_core::{
    error::{dom_exception_name, throw_dom_exception},
    webidl::{binary_string_to_bytes, bytes_to_binary_string, to_dom_string},
};

pub fn init(ctx: &Ctx) -> Result<()> {
    ctx.globals()
        .set("atob", Function::new(ctx.clone(), atob_impl))?;
    ctx.globals()
        .set("btoa", Function::new(ctx.clone(), btoa_impl))?;
    Ok(())
}

fn btoa_impl<'js>(ctx: Ctx<'js>, data: Value<'js>) -> Result<String> {
    let bytes = binary_string_to_bytes(&ctx, &data)?;
    Ok(general_purpose::STANDARD.encode(bytes))
}

fn atob_impl<'js>(ctx: Ctx<'js>, data: Value<'js>) -> Result<String> {
    let data = to_dom_string(&data)?;
    let mut cleaned: String = data.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if cleaned.len() % 4 == 1 {
        return throw_dom_exception(
            &ctx,
            dom_exception_name::INVALID_CHARACTER,
            "The string to be decoded is not correctly encoded",
        );
    }
    while cleaned.len() % 4 != 0 {
        cleaned.push('=');
    }
    match general_purpose::STANDARD.decode(cleaned) {
        Ok(decoded) => Ok(bytes_to_binary_string(&decoded)),
        Err(_) => throw_dom_exception(
            &ctx,
            dom_exception_name::INVALID_CHARACTER,
            "The string to be decoded is not correctly encoded",
        ),
    }
}
