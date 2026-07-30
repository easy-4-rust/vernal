//! 装配模块: annotation_bean_wiring_info_resolver。
use std::any::Any;
use std::sync::Arc;

/// AnnotationBeanWiringInfoResolver — 对应 Spring 装配组件。
#[derive(Debug, Clone, Default)]
pub struct AnnotationBeanWiringInfoResolver {
    // TODO: 添加字段
}

impl AnnotationBeanWiringInfoResolver {
    pub fn new() -> Self { Self::default() }
}
