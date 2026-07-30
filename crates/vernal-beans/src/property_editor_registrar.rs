//! property_editor_registrar — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// property_editor_registrar — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct property_editor_registrar {
    // TODO: 添加字段
}

impl property_editor_registrar {
    pub fn new() -> Self { Self::default() }
}
