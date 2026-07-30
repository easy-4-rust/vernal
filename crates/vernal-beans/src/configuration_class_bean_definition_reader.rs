//! ConfigurationClassBeanDefinitionReader — 配置类 Bean 定义读取器。
use crate::configuration_class::ConfigurationClass;

/// 配置类 Bean 定义读取器。
#[derive(Clone, Debug, Default)]
pub struct ConfigurationClassBeanDefinitionReader;
impl ConfigurationClassBeanDefinitionReader {
    pub fn new() -> Self { Self }
    pub fn load_bean_definitions(&self, _config: &ConfigurationClass) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0)
    }
}
