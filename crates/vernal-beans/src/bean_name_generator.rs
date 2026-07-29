//! BeanNameGenerator — Spring 风格的 Bean 名称生成器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanNameGenerator`。
//!
//! 为 Bean 定义生成唯一名称的策略接口。

use crate::bean_definition::BeanDefinition;

/// Spring 风格的 Bean 名称生成器 trait。
///
/// 对应 Spring 的 `BeanNameGenerator`。
///
/// 根据 Bean 定义为 Bean 生成唯一名称。
pub trait BeanNameGenerator: Send + Sync {
    /// 为 Bean 定义生成唯一名称。
    fn generate_bean_name(&self, definition: &dyn BeanDefinition) -> String;
}
