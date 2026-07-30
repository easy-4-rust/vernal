//! bean_definition_visitor — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanDefinitionVisitor — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionVisitor {
    // TODO: 添加字段
}

impl BeanDefinitionVisitor {
    pub fn new() -> Self { Self::default() }
}
