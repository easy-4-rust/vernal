//! XmlReaderContext — XML 读取器上下文。
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// XML 读取器上下文。
#[derive(Clone, Debug)]
pub struct XmlReaderContext {
    pub resource_description: String,
}
impl XmlReaderContext {
    pub fn new(resource_description: impl Into<String>) -> Self {
        Self { resource_description: resource_description.into() }
    }
    pub fn resource_description(&self) -> &str { &self.resource_description }
}
