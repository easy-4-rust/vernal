//! PropertiesBeanDefinitionReaderImpl — Properties Bean 定义读取器实现。
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// Properties Bean 定义读取器实现。
#[derive(Clone, Debug, Default)]
pub struct PropertiesBeanDefinitionReaderImpl;
impl PropertiesBeanDefinitionReaderImpl {
    pub fn new() -> Self { Self }
    pub fn load_bean_definitions(&self, _content: &str, _registry: &mut dyn BeanDefinitionRegistry) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }
}
