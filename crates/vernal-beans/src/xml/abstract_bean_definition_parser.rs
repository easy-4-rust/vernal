//! XML 解析模块: abstract_bean_definition_parser。
use std::any::Any;
use std::sync::Arc;

/// AbstractBeanDefinitionParser — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct AbstractBeanDefinitionParser {
    // TODO: 添加字段
}

impl AbstractBeanDefinitionParser {
    pub fn new() -> Self { Self::default() }
}
