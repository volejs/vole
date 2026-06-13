use rquickjs::{Class, Ctx, Result};

pub mod text_decoder;
pub mod text_encoder;

pub use text_decoder::TextDecoder;
pub use text_encoder::TextEncoder;

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<TextEncoder>::define(&ctx.globals())?;
    Class::<TextDecoder>::define(&ctx.globals())?;

    // Convert prototype methods to getters
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
            wrapGetter(TextEncoder.prototype, 'encoding');
            wrapGetter(TextDecoder.prototype, 'encoding');
            wrapGetter(TextDecoder.prototype, 'fatal');
            wrapGetter(TextDecoder.prototype, 'ignoreBOM');
        })();
    "#,
    )?;

    Ok(())
}
