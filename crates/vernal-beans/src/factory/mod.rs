//! factory — 对应 Spring beans.factory 包根模块。
//!
//! 包含 factory 子包：
//! - support: Bean 工厂支持实现
//! - annotation: 注解驱动配置
//! - aot: Ahead-of-Time 处理
//! - wiring: Bean 装配信息
//! - serviceloader: ServiceLoader 集成
//! - groovy: Groovy DSL 集成

pub mod support;
pub mod annotation;
pub mod aot;
pub mod wiring;
pub mod serviceloader;
pub mod groovy;
