//! registered_bean — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// RegisteredBean — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct RegisteredBean {
    // TODO: 添加字段
}

impl RegisteredBean {
    pub fn new() -> Self { Self::default() }
}
