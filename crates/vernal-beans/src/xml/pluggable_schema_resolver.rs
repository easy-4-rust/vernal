//! XML 解析模块: pluggable_schema_resolver。
use std::any::Any;
use std::sync::Arc;

/// PluggableSchemaResolver — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct PluggableSchemaResolver {
    // TODO: 添加字段
}

impl PluggableSchemaResolver {
    pub fn new() -> Self { Self::default() }
}
