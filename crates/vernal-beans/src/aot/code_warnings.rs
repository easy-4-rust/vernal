//! AOT 模块: code_warnings。
use std::any::Any;
use std::sync::Arc;

/// CodeWarnings — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct CodeWarnings {
    // TODO: 添加字段
}

impl CodeWarnings {
    pub fn new() -> Self { Self::default() }
}
