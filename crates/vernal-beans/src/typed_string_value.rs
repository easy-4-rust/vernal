//! typed_string_value — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// TypedStringValue — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct TypedStringValue {
    // TODO: 添加字段
}

impl TypedStringValue {
    pub fn new() -> Self { Self::default() }
}
