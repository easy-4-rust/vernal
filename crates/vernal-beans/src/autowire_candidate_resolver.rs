//! autowire_candidate_resolver — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// AutowireCandidateResolver — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct AutowireCandidateResolver {
    // TODO: 添加字段
}

impl AutowireCandidateResolver {
    pub fn new() -> Self { Self::default() }
}
