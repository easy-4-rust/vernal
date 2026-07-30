//! JoinPoint — Spring 风格的连接点。
use std::any::Any;
use std::fmt;

/// 连接点 trait。
pub trait JoinPoint: Send + Sync + fmt::Debug {
    fn get_method(&self) -> &str;
    fn get_args(&self) -> &[Box<dyn Any>];
}
