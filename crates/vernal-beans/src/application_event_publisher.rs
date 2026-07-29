//! ApplicationEventPublisher trait — Spring 风格的事件发布器。
use std::any::Any;
use std::sync::Arc;

/// 事件发布器 trait。
pub trait ApplicationEventPublisher: Send + Sync {
    fn publish_event(&self, event: Arc<dyn Any + Send + Sync>);
}
