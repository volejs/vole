//! Compatibility shim for the legacy `vole-raw-os` crate.
//!
//! Capability implementation is now split by domain:
//! - `vole-raw-env`
//! - `vole-raw-exec`
//! - `vole-raw-fs`
//! - `vole-raw-os` (kept for process-level helpers)
//!
//! Keep these exports so existing downstream code that still imports
//! `vole-raw-os` symbols continues to work.

use rquickjs::{Ctx, Result};

mod os;

pub use vole_raw_env::EnvRawModule as OsEnvRawModule;
pub use vole_raw_env::{load_dotenv, load_dotenv_with_path};
pub use vole_raw_exec::OsExecRawModule;
pub use vole_raw_fs::FsRawModule as OsFsRawModule;
pub use os::OsRawModule;

/// 兼容入口，保持一致的初始化行为。
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}
