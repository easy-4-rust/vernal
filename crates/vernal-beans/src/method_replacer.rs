//! MethodReplacer — Spring 风格的方法替换器 trait。
use std::any::Any;

/// Spring 风格的方法替换器 trait。
pub trait MethodReplacer: Send + Sync {
    fn reimplement(&self, target: &mut dyn Any, method: &str, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>>;
}
