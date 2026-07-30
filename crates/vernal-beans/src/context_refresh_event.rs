//! ContextRefreshEvent — 上下文刷新事件。
use crate::application_event::ApplicationEvent;

/// 上下文刷新事件。
#[derive(Clone, Debug)]
pub struct ContextRefreshEvent;
impl ApplicationEvent for ContextRefreshEvent {
    fn get_timestamp(&self) -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 }
    fn get_source(&self) -> &dyn std::any::Any { self }
}
