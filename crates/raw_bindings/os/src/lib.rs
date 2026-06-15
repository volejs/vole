use rquickjs::{Ctx, Result};

mod env;
mod exec;
mod fs;
mod os;

pub use env::EnvRawModule as OsEnvRawModule;
pub use env::{load_dotenv, load_dotenv_with_path};
pub use exec::OsExecRawModule;
pub use fs::FsRawModule as OsFsRawModule;
pub use os::OsRawModule;

/// 兼容入口，保持一致的初始化行为。
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}
