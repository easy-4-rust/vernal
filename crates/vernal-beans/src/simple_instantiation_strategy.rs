//! SimpleInstantiationStrategy — 简单实例化策略。
use crate::instantiation_strategy::InstantiationStrategy;
use std::any::Any;
use std::sync::Arc;

/// 简单实例化策略。
#[derive(Clone, Debug, Default)]
pub struct SimpleInstantiationStrategy;
impl SimpleInstantiationStrategy {
    pub fn new() -> Self { Self }
}
impl InstantiationStrategy for SimpleInstantiationStrategy {
    fn instantiate(&self, _bean_class: &str, _args: &[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("SimpleInstantiationStrategy: not implemented".into())
    }
}
