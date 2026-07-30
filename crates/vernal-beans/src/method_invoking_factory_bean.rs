//! method_invoking_factory_bean — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// method_invoking_factory_bean — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct method_invoking_factory_bean {
    // TODO: 添加字段
}

impl method_invoking_factory_bean {
    pub fn new() -> Self { Self::default() }
}
