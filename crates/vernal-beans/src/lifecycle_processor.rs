//! LifecycleProcessor trait — Spring 风格的生命周期处理器。
use std::fmt;

/// 生命周期处理器 trait。
pub trait LifecycleProcessor: Send + Sync + fmt::Debug {
    fn on_refresh(&self) {}
    fn on_close(&self) {}
}
