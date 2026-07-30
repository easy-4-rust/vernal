//! AOT 模块: aot_processing_exception。
use std::any::Any;
use std::sync::Arc;

/// AotProcessingException — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct AotProcessingException {
    // TODO: 添加字段
}

impl AotProcessingException {
    pub fn new() -> Self { Self::default() }
}
