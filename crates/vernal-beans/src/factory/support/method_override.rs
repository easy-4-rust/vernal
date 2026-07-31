//! MethodOverride — Spring 风格的方法覆盖 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodOverride`。
//!
//! 在 Spring 中，`MethodOverride` 是方法覆盖的抽象基类。
//! 它有两个主要子类：
//! - `LookupOverride` — 查找方法覆盖（`@Lookup`）
//! - `ReplaceOverride` — 替换方法覆盖（`replaced-method`）
//!
//! 方法覆盖允许在不修改原始 Bean 类的情况下替换方法行为。

use std::fmt;

/// 方法覆盖接口。
///
/// 对应 Spring 的 `MethodOverride`。
///
/// 定义方法覆盖的基本行为：获取方法名、判断是否适用。
pub trait MethodOverride: Send + Sync + fmt::Debug {
    /// 获取被覆盖的方法名。
    fn get_method_name(&self) -> &str;

    /// 判断此覆盖是否适用于当前上下文。
    ///
    /// 对应 Spring 的 `isOverloaded()` 的反义。
    /// 返回 `true` 表示此覆盖始终生效。
    fn is_applicable(&self) -> bool;
}

/// 简单的方法覆盖实现。
///
/// 用于测试和简单场景，不做特殊处理。
#[derive(Debug, Clone)]
pub struct SimpleMethodOverride {
    /// 方法名
    method_name: String,
    /// 是否适用
    applicable: bool,
}

impl SimpleMethodOverride {
    /// 创建默认适用的方法覆盖。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self { method_name: method_name.into(), applicable: true }
    }

    /// 创建指定适用性的方法覆盖。
    pub fn with_applicable(method_name: impl Into<String>, applicable: bool) -> Self {
        Self { method_name: method_name.into(), applicable }
    }
}

impl MethodOverride for SimpleMethodOverride {
    fn get_method_name(&self) -> &str { &self.method_name }
    fn is_applicable(&self) -> bool { self.applicable }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_method_override_default_applicable() {
        let mo = SimpleMethodOverride::new("doWork");
        assert_eq!(mo.get_method_name(), "doWork");
        assert!(mo.is_applicable());
    }

    #[test]
    fn simple_method_override_not_applicable() {
        let mo = SimpleMethodOverride::with_applicable("doWork", false);
        assert_eq!(mo.get_method_name(), "doWork");
        assert!(!mo.is_applicable());
    }

    #[test]
    fn debug_format() {
        let mo = SimpleMethodOverride::new("test");
        let debug = format!("{:?}", mo);
        assert!(debug.contains("test"));
    }
}
