//! LifecycleEvent — 生命周期事件。
use crate::application_event::ApplicationEvent;
use std::any::Any;

/// 生命周期事件枚举。
#[derive(Clone, Debug)]
pub enum LifecycleEvent {
    Started,
    Stopped,
    Refreshed,
    Closed,
}
impl ApplicationEvent for LifecycleEvent {
    fn get_timestamp(&self) -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 }
    fn get_source(&self) -> &dyn Any { self }
}
