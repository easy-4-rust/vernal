//! AnnotatedBeanDefinition — Spring 风格的注解驱动 Bean 定义 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotatedBeanDefinition`。
//!
//! 扩展 BeanDefinition，提供获取注解元数据的能力。
//! AnnotatedGenericBeanDefinition 实现了此 trait。

use crate::bean_definition::BeanDefinition;

/// Spring 风格的注解驱动 Bean 定义 trait。
///
/// 对应 Spring 的 `AnnotatedBeanDefinition`。
///
/// 扩展 `BeanDefinition`，提供获取注解元数据的能力。
/// 在 Rust 中，注解元数据映射为 `&'static str` 形式的类型名称。
pub trait AnnotatedBeanDefinition: BeanDefinition {
    /// 获取注解元数据（在 Rust 中返回类型名的静态字符串切片）。
    fn annotation_type_name(&self) -> &'static str;
}
