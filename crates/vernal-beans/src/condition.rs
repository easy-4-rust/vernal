//! Condition — 条件评估 trait。
use std::fmt;

/// 条件评估 trait。
pub trait Condition: Send + Sync + fmt::Debug {
    fn matches(&self, context: &ConditionContext) -> bool;
}

/// 条件上下文。
#[derive(Clone, Debug)]
pub struct ConditionContext {
    pub bean_type: Option<String>,
    pub bean_name: Option<String>,
}
impl ConditionContext {
    pub fn new() -> Self { Self { bean_type: None, bean_name: None } }
}
