//! AnnotationProcessor — 注解处理器。
use std::fmt;

/// 注解处理器 trait。
pub trait AnnotationProcessor: Send + Sync + fmt::Debug {
    fn process(&self, annotation_name: &str) -> Option<String>;
}
