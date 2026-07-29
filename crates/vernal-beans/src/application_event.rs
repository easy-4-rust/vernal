use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait ApplicationEvent: Send + Sync + Any {
    fn get_timestamp(&self) -> u128;
    fn get_source(&self) -> &(dyn Any + Send + Sync);
    fn as_any(&self) -> &dyn Any;
}

pub fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
