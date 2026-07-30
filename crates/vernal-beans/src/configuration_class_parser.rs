//! ConfigurationClassParser — 配置类解析器。
use crate::configuration_class::ConfigurationClass;

/// 配置类解析器。
#[derive(Clone, Debug, Default)]
pub struct ConfigurationClassParser;
impl ConfigurationClassParser {
    pub fn new() -> Self { Self }
    pub fn parse(&self, class_name: &str) -> ConfigurationClass {
        ConfigurationClass::new(class_name)
    }
}
