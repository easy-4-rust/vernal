//! bean_metadata_element — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanMetadataElement — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanMetadataElement {
    // TODO: 添加字段
}

impl BeanMetadataElement {
    pub fn new() -> Self { Self::default() }
}
