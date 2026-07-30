//! 解析模块: null_source_extractor。
use std::any::Any;
use std::sync::Arc;

/// NullSourceExtractor — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct NullSourceExtractor {
    // TODO: 添加字段
}

impl NullSourceExtractor {
    pub fn new() -> Self { Self::default() }
}
