//! AopContext — AOP 上下文。
use std::any::Any;
use std::sync::Arc;

/// AOP 上下文。
#[derive(Clone, Debug, Default)]
pub struct AopContext;
impl AopContext {
    pub fn current_proxy(&self) -> Option<Arc<dyn Any + Send + Sync>> { None }
}
