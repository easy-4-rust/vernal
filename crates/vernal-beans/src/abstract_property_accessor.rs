//! abstract_property_accessor — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// AbstractPropertyAccessor — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct AbstractPropertyAccessor {
    // TODO: 添加字段
}

impl AbstractPropertyAccessor {
    pub fn new() -> Self { Self::default() }
}
