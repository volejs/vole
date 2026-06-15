//! Fetch 内部 HTTP client 边界。
//!
//! `llrt_fetch` 原始实现依赖 `llrt_http`。vole 迁移时不把 `llrt_http` 作为外部
//! 依赖保留，而是把 fetch 需要的 client 能力收敛在本模块下，再按需沉淀。
