//! `vole-core` 是 vole 的底层共享能力层。
//!
//! 这个 crate 不负责创建 VM、注册全局 Web API，也不持有 CLI 或 runtime
//! 生命周期。它只提供各个 `global_bindings/*`、`raw_bindings/*` 和
//! `vole-runtime` 可以共同复用的 rquickjs / WebIDL / JS 值处理工具。
//!
//! 设计边界：
//! - 这里可以放“无业务语义”的底层能力，例如 primordials、BufferSource、
//!   Promise helper、WebIDL 参数转换、JS Error 转换、编码算法等。
//! - 这里不应该放具体 Web API 的状态机，例如 Streams、Fetch、Crypto 的
//!   业务逻辑。
//! - 这里不应该放 runtime 策略，例如全局错误分发、模块加载策略、CLI 行为、
//!   权限模型或资源生命周期管理。
//!
//! 使用注意：
//! - 所有模块都应尽量保持“纯 helper”属性，避免反向依赖 `vole-runtime`。
//! - 如果某个 helper 需要触发 `error` / `unhandledrejection` / `reportError`，
//!   它通常不适合放在这里，应放到 runtime 层。
//! - 从 LLRT 迁移来的代码需要逐步消除 LLRT 命名和运行时假设，保留的是
//!   设计思想和可复用实现，不是 LLRT 的架构边界。

pub mod array_buffer;
pub mod bytes;
pub mod class;
pub mod encoding;
pub mod error;
pub mod json;
pub mod latch;
pub mod mc_oneshot;
pub mod object;
pub mod option;
pub mod policy;
pub mod primordials;
pub mod result;
pub mod promise;
pub mod queue;
pub mod string;
pub mod time;
pub mod until;
pub mod webidl;

/// 跨模块共享的错误消息常量。
///
/// 这里只放会被多个底层模块使用、且不带具体 Web API 语义的消息。
/// 如果消息只属于某个 API，例如 Streams 的状态机错误，应保留在对应
/// binding crate 内部，避免 core 逐渐变成业务错误消息集合。
pub mod error_messages {
    pub const ERROR_MSG_ARRAY_BUFFER_DETACHED: &str = "ArrayBuffer is detached";
}
