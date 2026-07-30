//! bean_definition_overriding — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionOverriding — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionOverriding {
    // TODO: 添加字段
}

impl BeanDefinitionOverriding {
    pub fn new() -> Self { Self::default() }
}
