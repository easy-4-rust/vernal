//! bean_metadata_attribute — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanMetadataAttribute — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanMetadataAttribute {
    // TODO: 添加字段
}

impl BeanMetadataAttribute {
    pub fn new() -> Self { Self::default() }
}
