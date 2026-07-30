//! argument_converting_method_invoker — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// ArgumentConvertingMethodInvoker — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct ArgumentConvertingMethodInvoker {
    // TODO: 添加字段
}

impl ArgumentConvertingMethodInvoker {
    pub fn new() -> Self { Self::default() }
}
