//! property_accessor_factory — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// PropertyAccessorFactory — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct PropertyAccessorFactory {
    // TODO: 添加字段
}

impl PropertyAccessorFactory {
    pub fn new() -> Self { Self::default() }
}
