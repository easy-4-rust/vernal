//! Advisor — Spring 风格的通知器。
use std::fmt;

/// 通知器 trait。
pub trait Advisor: Send + Sync + fmt::Debug {
    fn get_advice(&self) -> &dyn Advice;
    fn is_per_instance(&self) -> bool { false }
}

/// 通知 trait。
pub trait Advice: Send + Sync + fmt::Debug {}
