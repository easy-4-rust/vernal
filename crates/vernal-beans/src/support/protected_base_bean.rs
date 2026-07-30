//! 支撑模块: protected_base_bean。
use std::any::Any;
use std::sync::Arc;

/// ProtectedBaseBean — 对应 Spring support 组件。
#[derive(Debug, Clone, Default)]
pub struct ProtectedBaseBean {
    // TODO: 添加字段
}

impl ProtectedBaseBean {
    pub fn new() -> Self { Self::default() }
}
