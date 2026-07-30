//! 解析模块: bean_definition_parsing_exception。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionParsingException — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionParsingException {
    // TODO: 添加字段
}

impl BeanDefinitionParsingException {
    pub fn new() -> Self { Self::default() }
}
