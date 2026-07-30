//! 注解模块: class_path_scanning_candidate_component_provider。
use std::any::Any;
use std::sync::Arc;

/// ClassPathScanningCandidateComponentProvider — 对应 Spring 注解组件。
#[derive(Debug, Clone, Default)]
pub struct ClassPathScanningCandidateComponentProvider {
    // TODO: 添加字段
}

impl ClassPathScanningCandidateComponentProvider {
    pub fn new() -> Self { Self::default() }
}
