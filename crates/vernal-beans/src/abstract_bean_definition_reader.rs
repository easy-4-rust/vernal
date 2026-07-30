//! AbstractBeanDefinitionReader — 抽象 Bean 定义读取器。
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// 抽象 Bean 定义读取器。
#[derive(Clone, Debug, Default)]
pub struct AbstractBeanDefinitionReader;
impl AbstractBeanDefinitionReader {
    pub fn new() -> Self { Self }
    pub fn load_bean_definitions(&self, _content: &str, _registry: &mut dyn BeanDefinitionRegistry) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }
}
