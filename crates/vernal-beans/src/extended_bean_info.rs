//! extended_bean_info — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// ExtendedBeanInfo — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct ExtendedBeanInfo {
    // TODO: 添加字段
}

impl ExtendedBeanInfo {
    pub fn new() -> Self { Self::default() }
}
