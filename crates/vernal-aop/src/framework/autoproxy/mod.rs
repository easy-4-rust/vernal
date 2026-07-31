//! 自动代理模块。
//!
//! 对应 spring-aop `framework/autoproxy` 包。

pub mod target_source_creator;

// Re-export
pub use target_source_creator::{FnTargetSourceCreator, TargetSourceCreator};
