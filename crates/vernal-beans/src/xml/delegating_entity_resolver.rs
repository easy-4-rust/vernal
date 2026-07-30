//! XML 解析模块: delegating_entity_resolver。
use std::any::Any;
use std::sync::Arc;

/// DelegatingEntityResolver — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct DelegatingEntityResolver {
    // TODO: 添加字段
}

impl DelegatingEntityResolver {
    pub fn new() -> Self { Self::default() }
}
