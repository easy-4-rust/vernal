use crate::application_event::{ApplicationEvent, current_timestamp_millis};
use std::any::Any;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEventKind {
    ContextRefreshedEvent,
    ContextStartedEvent,
    ContextStoppedEvent,
    ContextClosedEvent,
}
#[derive(Debug)]
pub struct LifecycleEvent {
    kind: LifecycleEventKind,
    source: Box<dyn Any + Send + Sync>,
    timestamp: u128,
}
impl LifecycleEvent {
    pub fn new<T: Any + Send + Sync>(kind: LifecycleEventKind, source: T) -> Self {
        Self {
            kind,
            source: Box::new(source),
            timestamp: current_timestamp_millis(),
        }
    }
    pub fn kind(&self) -> LifecycleEventKind {
        self.kind
    }
}
impl ApplicationEvent for LifecycleEvent {
    fn get_timestamp(&self) -> u128 {
        self.timestamp
    }
    fn get_source(&self) -> &(dyn Any + Send + Sync) {
        self.source.as_ref()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
