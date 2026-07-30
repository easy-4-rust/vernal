//! 适配器模块。
//!
//! 对应 spring-aop `framework/adapter` 包。

pub mod advisor_adapter;

// Re-export
pub use advisor_adapter::{AdvisorAdapter, AdvisorAdapterRegistry, DefaultAdvisorAdapterRegistry};
