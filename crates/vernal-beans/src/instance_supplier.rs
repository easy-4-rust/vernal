//! instance_supplier — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// InstanceSupplier — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct InstanceSupplier {
    // TODO: 添加字段
}

impl InstanceSupplier {
    pub fn new() -> Self { Self::default() }
}
