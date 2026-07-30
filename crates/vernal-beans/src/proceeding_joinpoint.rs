//! ProceedingJoinPoint — 可继续执行的连接点。
use crate::joinpoint::JoinPoint;

/// 可继续执行的连接点 trait。
pub trait ProceedingJoinPoint: JoinPoint {
    fn proceed(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}
