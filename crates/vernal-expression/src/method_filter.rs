//! 方法过滤器 trait（对标 Spring `MethodFilter`）。
//!
//! Spring 用 `MethodFilter` 限定 `ReflectiveMethodResolver` 在某类型上能解析哪些方法。
//! 在 Rust 中通过 `dyn Fn` 或 trait 对象实现。

use crate::method_resolver::MethodResolver;

/// 方法过滤器（对标 `org.springframework.expression.MethodFilter`）。
pub trait MethodFilter: Send + Sync {
    /// 决定是否允许该方法（名称 + 参数类型匹配）。
    #[must_use]
    fn filter(&self, method_name: &str, argument_types: &[&str]) -> bool;
}

/// `ReflectiveMethodResolver` 注册表（按类型）。
///
/// 对标 Spring `StandardEvaluationContext.registerMethodFilter(Class<?>, MethodFilter)`。
pub type MethodFilterRegistry = std::collections::HashMap<std::any::TypeId, Vec<Box<dyn MethodFilter>>>;
