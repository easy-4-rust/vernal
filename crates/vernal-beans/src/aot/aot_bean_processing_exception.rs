//! AOT 模块: aot_bean_processing_exception。
use std::any::Any;
use std::sync::Arc;

/// AotBeanProcessingException — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct AotBeanProcessingException {
    // TODO: 添加字段
}

impl AotBeanProcessingException {
    pub fn new() -> Self { Self::default() }
}
