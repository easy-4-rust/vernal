//! UI 集成模块 — 对标 `org.springframework.ui`。
//!
//! 包含：
//! - `freemarker`：FreeMarker 模板引擎适配（tera 后端）

#[cfg(feature = "freemarker")]
pub mod freemarker;
