//! 装配模块: class_name_bean_wiring_info_resolver。
use std::any::Any;
use std::sync::Arc;

/// ClassNameBeanWiringInfoResolver — 对应 Spring 装配组件。
#[derive(Debug, Clone, Default)]
pub struct ClassNameBeanWiringInfoResolver {
    // TODO: 添加字段
}

impl ClassNameBeanWiringInfoResolver {
    pub fn new() -> Self { Self::default() }
}
