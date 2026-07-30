//! 装配模块: bean_wiring_info。
use std::any::Any;
use std::sync::Arc;

/// BeanWiringInfo — 对应 Spring 装配组件。
#[derive(Debug, Clone, Default)]
pub struct BeanWiringInfo {
    // TODO: 添加字段
}

impl BeanWiringInfo {
    pub fn new() -> Self { Self::default() }
}
