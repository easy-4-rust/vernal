//! ApplicationEvent — Spring 风格的应用事件。
use std::any::Any;
use std::fmt;

/// 应用事件 trait。
pub trait ApplicationEvent: Send + Sync + fmt::Debug {
    fn get_timestamp(&self) -> u64;
    fn get_source(&self) -> &dyn Any;
}

/// 通用应用事件。
#[derive(Clone, Debug)]
pub struct GenericApplicationEvent {
    pub timestamp: u64,
    pub source_type: String,
}
impl GenericApplicationEvent {
    pub fn new() -> Self { Self { timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64, source_type: String::new() } }
}
