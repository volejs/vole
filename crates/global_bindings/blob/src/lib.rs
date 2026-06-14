use rquickjs::{Class, Ctx, Result};
use vole_core::{
    primordials::{BasePrimordials, Primordial},
    promise::PromisePrimordials,
};

pub mod blob;
pub mod file;

pub use blob::Blob;
pub use file::File;

pub fn init(ctx: &Ctx) -> Result<()> {
    BasePrimordials::init(ctx)?;
    PromisePrimordials::init(ctx)?;

    Class::<Blob>::define(&ctx.globals())?;
    Class::<File>::define(&ctx.globals())?;

    Ok(())
}
