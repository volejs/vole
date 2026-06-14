use rquickjs::{Class, Ctx, Result};

pub mod text_decoder;
pub mod text_encoder;

pub use text_decoder::TextDecoder;
pub use text_encoder::TextEncoder;

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<TextEncoder>::define(&ctx.globals())?;
    Class::<TextDecoder>::define(&ctx.globals())?;
    Ok(())
}
