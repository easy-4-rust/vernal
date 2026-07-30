//! ErrorHandler — 错误处理器 trait。
use std::fmt;

/// 错误处理器 trait。
pub trait ErrorHandler: Send + Sync + fmt::Debug {
    fn handle_error(&self, error: &dyn std::error::Error) -> bool;
}
