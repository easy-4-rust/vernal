//! BeanDefinitionReader — Spring 风格的 Bean 定义读取器 trait。
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionReader`。
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// Spring 风格的 Bean 定义读取器 trait。
pub trait BeanDefinitionReader {
    fn registry(&self) -> &dyn BeanDefinitionRegistry;
    fn load_bean_definitions(&mut self, source: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}
