//! 解析模块: bean_entry。
use std::any::Any;
use std::sync::Arc;

/// BeanEntry — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct BeanEntry {
    // TODO: 添加字段
}

impl BeanEntry {
    pub fn new() -> Self { Self::default() }
}
