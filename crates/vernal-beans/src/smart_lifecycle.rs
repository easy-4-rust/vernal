//! SmartLifecycle — Spring 风格的智能生命周期。
use crate::lifecycle::Lifecycle;

/// 智能生命周期 trait。
pub trait SmartLifecycle: Lifecycle {
    fn get_phase(&self) -> i32 { 0 }
    fn is_auto_startup(&self) -> bool { true }
}
