//! XML 解析模块: bean_definition_decorator。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionDecorator — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionDecorator {
    // TODO: 添加字段
}

impl BeanDefinitionDecorator {
    pub fn new() -> Self { Self::default() }
}
