//! simple_bean_info_factory — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// SimpleBeanInfoFactory — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct SimpleBeanInfoFactory {
    // TODO: 添加字段
}

impl SimpleBeanInfoFactory {
    pub fn new() -> Self { Self::default() }
}
