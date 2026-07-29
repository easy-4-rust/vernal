//! Lifecycle trait — Spring 风格的生命周期。
use std::fmt;

/// 生命周期 trait。
pub trait Lifecycle: Send + Sync + fmt::Debug {
    fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    fn is_running(&self) -> bool { false }
}
