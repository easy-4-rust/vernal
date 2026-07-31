//! NullBean — Spring 风格的空 Bean 标记。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.NullBean`。
//!
//! 在 Spring 中，`NullBean` 用于表示一个显式的 `null` Bean 值。
//! 当 XML 配置中使用 `<null/>` 标签时，Spring 会创建一个 `NullBean` 实例
//! 来区分"未设置"和"显式设为 null"的场景。
//!
//! ## 设计说明
//!
//! 在 Rust 中，`Option::None` 通常用于表示缺失值。
//! `NullBean` 在 vernal 中作为显式空值标记，用于 Bean 工厂中
//! 需要区分 `Some(none_marker)` 和 `None` 的场景。

use std::fmt;

/// 空 Bean 标记。
///
/// 对应 Spring 的 `NullBean`。
///
/// 表示一个显式的空值 Bean，区别于未注册或未初始化的 Bean。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NullBean;

impl NullBean {
    /// 创建空 Bean 实例。
    pub fn new() -> Self {
        Self
    }

    /// 获取单例空 Bean 引用。
    ///
    /// 所有 `NullBean` 实例等价，此方法返回共享实例。
    pub fn instance() -> &'static NullBean {
        &NullBean
    }
}

impl Default for NullBean {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NullBean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

/// 判断一个值是否为 `NullBean`。
///
/// 在 Bean 工厂中用于检查 Bean 值是否为显式空值。
pub fn is_null_bean(value: &dyn std::any::Any) -> bool {
    value.downcast_ref::<NullBean>().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_bean_display() {
        let bean = NullBean::new();
        assert_eq!(format!("{}", bean), "null");
    }

    #[test]
    fn null_bean_instances_are_equal() {
        let a = NullBean::new();
        let b = NullBean::new();
        assert_eq!(a, b);
    }

    #[test]
    fn is_null_bean_detects_null() {
        let null_bean: Box<dyn std::any::Any> = Box::new(NullBean::new());
        assert!(is_null_bean(null_bean.as_ref()));

        let regular: Box<dyn std::any::Any> = Box::new(42_i32);
        assert!(!is_null_bean(regular.as_ref()));
    }

    #[test]
    fn null_bean_instance_returns_same_reference() {
        let a = NullBean::instance();
        let b = NullBean::instance();
        assert!(std::ptr::eq(a, b));
    }
}
