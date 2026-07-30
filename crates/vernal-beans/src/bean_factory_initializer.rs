//! bean_factory_initializer — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanFactoryInitializer — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanFactoryInitializer {
    // TODO: 添加字段
}

impl BeanFactoryInitializer {
    pub fn new() -> Self { Self::default() }
}
