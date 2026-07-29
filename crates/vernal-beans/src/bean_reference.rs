//! BeanReference — Spring 风格的对另一个 Bean 的引用标记 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanReference`。
//!
//! 用于在 Bean 定义中标记一个值是另一个 Bean 的引用。

use std::any::Any;

/// Spring 风格的 Bean 引用标记 trait。
///
/// 对应 Spring 的 `BeanReference`。
///
/// 当一个 Bean 定义中的属性值或构造参数值是一个对另一个 Bean 的引用时，
/// 使用此 trait 标记。`BeanDefinitionValueResolver` 遇到此类型时会解析
/// 为实际的 Bean 实例。
pub trait BeanReference: Send + Sync + std::fmt::Debug {
    /// 获取目标 Bean 的名称。
    ///
    /// 对应 Spring 的 `String getBeanName()`。
    fn get_bean_name(&self) -> &str;

    /// 获取引用的来源（源对象）。
    ///
    /// 对应 Spring 的 `Object getSource()`。
    ///
    /// 返回 `None` 表示没有来源信息。
    fn get_source(&self) -> Option<&dyn Any>;
}
