//! AOT 模块: bean_instance_supplier。
use std::any::Any;
use std::sync::Arc;

/// BeanInstanceSupplier — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanInstanceSupplier {
    // TODO: 添加字段
}

impl BeanInstanceSupplier {
    pub fn new() -> Self { Self::default() }
}
