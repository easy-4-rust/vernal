//! XML 解析模块: abstract_single_bean_definition_parser。
use std::any::Any;
use std::sync::Arc;

/// AbstractSingleBeanDefinitionParser — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct AbstractSingleBeanDefinitionParser {
    // TODO: 添加字段
}

impl AbstractSingleBeanDefinitionParser {
    pub fn new() -> Self { Self::default() }
}
