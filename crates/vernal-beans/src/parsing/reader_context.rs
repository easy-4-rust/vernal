//! 解析模块: reader_context。
use std::any::Any;
use std::sync::Arc;

/// ReaderContext — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct ReaderContext {
    // TODO: 添加字段
}

impl ReaderContext {
    pub fn new() -> Self { Self::default() }
}
