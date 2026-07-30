//! runtime_bean_name_reference — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// RuntimeBeanNameReference — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct RuntimeBeanNameReference {
    // TODO: 添加字段
}

impl RuntimeBeanNameReference {
    pub fn new() -> Self { Self::default() }
}
