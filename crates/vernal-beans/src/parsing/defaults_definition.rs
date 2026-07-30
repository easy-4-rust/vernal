//! 解析模块: defaults_definition。
use std::any::Any;
use std::sync::Arc;

/// DefaultsDefinition — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct DefaultsDefinition {
    // TODO: 添加字段
}

impl DefaultsDefinition {
    pub fn new() -> Self { Self::default() }
}
