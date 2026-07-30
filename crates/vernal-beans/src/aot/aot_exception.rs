//! AOT 模块: aot_exception。
use std::any::Any;
use std::sync::Arc;

/// AotException — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct AotException {
    // TODO: 添加字段
}

impl AotException {
    pub fn new() -> Self { Self::default() }
}
