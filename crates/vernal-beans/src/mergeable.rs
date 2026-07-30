//! mergeable — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// Mergeable — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct Mergeable {
    // TODO: 添加字段
}

impl Mergeable {
    pub fn new() -> Self { Self::default() }
}
