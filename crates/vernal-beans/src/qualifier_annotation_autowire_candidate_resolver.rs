//! qualifier_annotation_autowire_candidate_resolver — 对应 Java 类或 Spring 组件。
use std::any::Any;
use std::sync::Arc;

/// QualifierAnnotationAutowireCandidateResolver — 对应 Spring 组件。
#[derive(Debug, Clone, Default)]
pub struct QualifierAnnotationAutowireCandidateResolver {
    // TODO: 添加字段
}

impl QualifierAnnotationAutowireCandidateResolver {
    pub fn new() -> Self { Self::default() }
}
