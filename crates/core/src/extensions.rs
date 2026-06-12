use rquickjs::{Ctx, Result};

/// 扩展注册表。
///
/// 参考 LLRT 的 `ModuleBuilder` 与 rquickjs-extension 的 `ExtensionBuilder` 设计，
/// 提供一个轻量级的声明式扩展注册框架。
///
/// 每个扩展提供一个 `fn(&Ctx) -> Result<()>` 初始化函数，
/// 由 `ExtensionRegistry` 在 VM 启动时统一调用。
///
/// # Example
///
/// ```ignore
/// ExtensionRegistry::new()
///     .with(crate::raw_bindings::console::init)
///     .with(crate::global_bindings::timers::init)
///     .init(ctx)?;
/// ```
pub struct ExtensionRegistry {
    inits: Vec<fn(&Ctx) -> Result<()>>,
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self { inits: Vec::new() }
    }

    /// 注册一个全局初始化函数。
    pub fn with(mut self, init: fn(&Ctx) -> Result<()>) -> Self {
        self.inits.push(init);
        self
    }

    /// 一次性执行所有已注册的初始化函数。
    pub fn init(self, ctx: &Ctx) -> Result<()> {
        for init in self.inits {
            init(ctx)?;
        }
        Ok(())
    }
}
