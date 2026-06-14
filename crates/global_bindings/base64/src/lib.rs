use base64::{engine::general_purpose, Engine as _};
use rquickjs::{Ctx, Function};

pub fn init(ctx: &Ctx) -> Result<(), rquickjs::Error> {
    ctx.globals()
        .set("atob", Function::new(ctx.clone(), atob_impl))?;
    ctx.globals()
        .set("btoa", Function::new(ctx.clone(), btoa_impl))?;
    Ok(())
}

fn btoa_impl(data: String) -> Result<String, rquickjs::Error> {
    // btoa 输入必须是 Latin1（0-255），否则抛出 InvalidCharacterError
    if data.bytes().any(|b| b > 127) {
        return Err(rquickjs::Error::Exception);
    }
    Ok(general_purpose::STANDARD.encode(data))
}

fn atob_impl(data: String) -> Result<String, rquickjs::Error> {
    // atob 忽略输入中的空白字符
    let cleaned: String = data.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    let decoded = general_purpose::STANDARD
        .decode(cleaned)
        .map_err(|_| rquickjs::Error::Exception)?;
    // atob 返回 Latin1 字符串（字节直接转字符串）
    Ok(decoded.into_iter().map(|b| b as char).collect())
}
