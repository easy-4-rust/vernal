//! method_invocation_exception — 对应 Java 类：org.springframework.beans.MethodInvocationException。
//!
//! 对应 Spring beans 包。

use std::any::Any;
use std::sync::Arc;

/// MethodInvocationException — Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct MethodInvocationException {
    // TODO: 添加字段
}

impl MethodInvocationException {
    pub fn new() -> Self { Self::default() }
}
