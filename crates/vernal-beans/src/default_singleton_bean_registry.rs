//! default_singleton_bean_registry — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// DefaultSingletonBeanRegistry — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct DefaultSingletonBeanRegistry {
    // TODO: 添加字段
}

impl DefaultSingletonBeanRegistry {
    pub fn new() -> Self { Self::default() }
}
