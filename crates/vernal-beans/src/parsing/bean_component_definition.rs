//! 解析模块: bean_component_definition。
use std::any::Any;
use std::sync::Arc;

/// BeanComponentDefinition — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanComponentDefinition {
    // TODO: 添加字段
}

impl BeanComponentDefinition {
    pub fn new() -> Self { Self::default() }
}
