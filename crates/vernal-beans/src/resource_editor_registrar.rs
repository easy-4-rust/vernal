//! resource_editor_registrar — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// ResourceEditorRegistrar — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct ResourceEditorRegistrar {
    // TODO: 添加字段
}

impl ResourceEditorRegistrar {
    pub fn new() -> Self { Self::default() }
}
