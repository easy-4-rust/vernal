//! annotation_bean_wiring_info_resolver — 对应 Java 类：org.springframework.beans.factory.wiring.AnnotationBeanWiringInfoResolver。
//!
//! 对应 Spring beans.factory.wiring 包。

use std::any::Any;
use std::sync::Arc;

/// AnnotationBeanWiringInfoResolver — Spring factory.wiring 组件。
#[derive(Debug, Clone, Default)]
pub struct AnnotationBeanWiringInfoResolver {
    // TODO: 添加字段
}

impl AnnotationBeanWiringInfoResolver {
    pub fn new() -> Self { Self::default() }
}
