//! property_values — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// PropertyValues — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct PropertyValues {
    // TODO: 添加字段
}

impl PropertyValues {
    pub fn new() -> Self { Self::default() }
}
