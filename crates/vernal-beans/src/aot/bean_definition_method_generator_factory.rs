//! AOT 模块: bean_definition_method_generator_factory。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionMethodGeneratorFactory — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionMethodGeneratorFactory {
    // TODO: 添加字段
}

impl BeanDefinitionMethodGeneratorFactory {
    pub fn new() -> Self { Self::default() }
}
