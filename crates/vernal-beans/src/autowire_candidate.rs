//! autowire_candidate — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// AutowireCandidate — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct AutowireCandidate {
    // TODO: 添加字段
}

impl AutowireCandidate {
    pub fn new() -> Self { Self::default() }
}
