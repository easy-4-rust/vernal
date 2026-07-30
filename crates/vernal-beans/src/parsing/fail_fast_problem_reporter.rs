//! 解析模块: fail_fast_problem_reporter。
use std::any::Any;
use std::sync::Arc;

/// FailFastProblemReporter — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct FailFastProblemReporter {
    // TODO: 添加字段
}

impl FailFastProblemReporter {
    pub fn new() -> Self { Self::default() }
}
