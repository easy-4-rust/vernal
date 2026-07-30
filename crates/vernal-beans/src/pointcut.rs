//! Pointcut — Spring 风格的切入点。
use std::fmt;

/// 切入点 trait。
pub trait Pointcut: Send + Sync + fmt::Debug {
    fn matches(&self, method_name: &str) -> bool;
}
