//! type_converter_support — 对应 Java 类：org.springframework.beans.TypeConverterSupport。
//!
//! 对应 Spring beans 包。

use std::any::Any;
use std::sync::Arc;

/// TypeConverterSupport — Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct TypeConverterSupport {
    // TODO: 添加字段
}

impl TypeConverterSupport {
    pub fn new() -> Self { Self::default() }
}
