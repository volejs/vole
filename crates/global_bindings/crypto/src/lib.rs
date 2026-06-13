use rquickjs::{Class, Ctx, Result};

use crate::classes::{Crypto, CryptoKey, SubtleCrypto};

pub(crate) mod cipher;
pub(crate) mod classes;
pub(crate) mod digest;
pub(crate) mod generate_key;
pub(crate) mod import_export;
pub(crate) mod key_material;
pub(crate) mod random;
pub(crate) mod signature;
pub(crate) mod utils;

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    Class::<Crypto>::define(&globals)?;
    Class::<SubtleCrypto>::define(&globals)?;
    Class::<CryptoKey>::define(&globals)?;

    // Turn the CryptoKey prototype helper methods into real getters.
    ctx.eval::<(), _>(
        r#"
        (function() {
            function wrapGetter(proto, name) {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: function() { return fn.call(this); },
                    enumerable: true,
                    configurable: true
                });
            }
            wrapGetter(CryptoKey.prototype, "type");
            wrapGetter(CryptoKey.prototype, "extractable");
            wrapGetter(CryptoKey.prototype, "algorithm");
            wrapGetter(CryptoKey.prototype, "usages");
            wrapGetter(CryptoKey.prototype, "_id");
        })();
    "#,
    )?;

    let subtle = Class::instance(ctx.clone(), SubtleCrypto)?;
    let crypto = Class::instance(ctx.clone(), Crypto)?;
    crypto.set("subtle", subtle)?;
    globals.set("crypto", crypto)?;

    Ok(())
}
