//! XML 解析模块: abstract_simple_bean_definition_parser。
use std::any::Any;
use std::sync::Arc;

/// AbstractSimpleBeanDefinitionParser — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct AbstractSimpleBeanDefinitionParser {
    // TODO: 添加字段
}

impl AbstractSimpleBeanDefinitionParser {
    pub fn new() -> Self { Self::default() }
}
