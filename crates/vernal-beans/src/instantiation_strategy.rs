//! InstantiationStrategy — 实例化策略。
use std::any::Any;
use std::sync::Arc;

/// 实例化策略 trait。
pub trait InstantiationStrategy: Send + Sync {
    fn instantiate(&self, bean_class: &str, constructor_args: &[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}
