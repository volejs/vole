//! JSON 解析、序列化与字符串转义 helper。
//!
//! 这个模块迁移自 LLRT 的 `llrt_json`，但只作为 `vole-core` 的底层能力使用，
//! 不负责覆盖或重新定义全局 `JSON.parse` / `JSON.stringify`。
//!
//! 设计用途：
//! - 给需要高性能 JSON 处理的 binding 或 runtime 模块复用，例如 HTTP、fetch、
//!   structured data bridge。
//! - 保留 QuickJS 值与 Rust JSON 编码之间的转换逻辑，减少各模块重复实现。
//! - 将 JSON 字符串 escaping 作为独立 helper 复用，尤其是处理 QuickJS 字符串中
//!   可能出现的 WTF-8 lone surrogate 场景。
//!
//! 注意事项：
//! - 这里不处理全局对象安装，也不决定异常如何分发到 `reportError` 或全局事件。
//! - `parse` 使用 `simd-json`，输入会被复制成可变 buffer；调用方不要假设零拷贝。
//! - `stringify` 会访问 JS 对象、调用 `toJSON`、读取属性和数组元素，因此必须在
//!   有效的 rquickjs `Ctx` 生命周期内使用。

pub mod escape;
pub mod parse;
pub mod stringify;
