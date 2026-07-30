//! XML 解析模块: parser_context。
use std::any::Any;
use std::sync::Arc;

/// ParserContext — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct ParserContext {
    // TODO: 添加字段
}

impl ParserContext {
    pub fn new() -> Self { Self::default() }
}
