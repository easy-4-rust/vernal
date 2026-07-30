//! XML 解析模块: bean_definition_parser。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionParser — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionParser {
    // TODO: 添加字段
}

impl BeanDefinitionParser {
    pub fn new() -> Self { Self::default() }
}
