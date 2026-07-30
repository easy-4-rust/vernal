//! AopProxy — AOP 代理。
use std::any::Any;
use std::sync::Arc;

/// AOP 代理 trait。
pub trait AopProxy: Send + Sync {
    fn get_proxy(&self) -> Arc<dyn Any + Send + Sync>;
    fn is_proxy(&self) -> bool { true }
}
