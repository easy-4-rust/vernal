//! ApplicationListener trait — Spring 风格的事件监听器。
use std::any::Any;
use std::sync::Arc;

/// 事件监听器 trait。
pub trait ApplicationListener: Send + Sync {
    fn on_application_event(&self, event: Arc<dyn Any + Send + Sync>);
}
