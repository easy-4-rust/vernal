//! XML 解析模块: bean_definition_parser_delegate。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionParserDelegate — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionParserDelegate {
    // TODO: 添加字段
}

impl BeanDefinitionParserDelegate {
    pub fn new() -> Self { Self::default() }
}
