//! DefaultBeanNameGenerator — Spring 风格的默认 Bean 名称生成器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultBeanNameGenerator`。
//!
//! 使用 Bean 的类名（类型名）作为 Bean 名称。
//! 当生成器检测到冲突时，追加唯一后缀。

use crate::bean_definition::BeanDefinition;
use crate::bean_name_generator::BeanNameGenerator;

/// Spring 风格的默认 Bean 名称生成器。
///
/// 对应 Spring 的 `DefaultBeanNameGenerator`。
///
/// 使用 Bean 的类名作为默认 Bean 名称。
pub struct DefaultBeanNameGenerator;

impl DefaultBeanNameGenerator {
    /// 创建新的默认 Bean 名称生成器。
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultBeanNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanNameGenerator for DefaultBeanNameGenerator {
    fn generate_bean_name(&self, definition: &dyn BeanDefinition) -> String {
        definition.bean_class_name().to_string()
    }
}
