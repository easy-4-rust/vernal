//! MethodInterceptor — 方法拦截器。
use std::any::Any;
use std::fmt;

/// 方法拦截器 trait。
pub trait MethodInterceptor: Send + Sync + fmt::Debug {
    fn invoke(&self, invocation: &dyn Invocation) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 调用上下文 trait。
pub trait Invocation: Send + Sync {
    fn get_method(&self) -> &str;
    fn get_arguments(&self) -> Vec<Box<dyn Any>>;
    fn proceed(&self) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}
