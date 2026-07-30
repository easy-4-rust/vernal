//! Aspect — Spring 风格的切面。
use std::fmt;

/// 切面 trait。
pub trait Aspect: Send + Sync + fmt::Debug {
    fn name(&self) -> &str;
}
