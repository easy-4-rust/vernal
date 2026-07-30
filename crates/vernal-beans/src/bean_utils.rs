//! bean_utils — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// BeanUtils — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanUtils {
    // TODO: 添加字段
}

impl BeanUtils {
    pub fn new() -> Self { Self::default() }
}
