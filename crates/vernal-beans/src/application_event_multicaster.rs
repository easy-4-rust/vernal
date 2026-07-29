//! ApplicationEventMulticaster trait — Spring 风格的事件广播器。
use crate::application_listener::ApplicationListener;
use std::any::Any;
use std::sync::Arc;

/// 事件广播器 trait。
pub trait ApplicationEventMulticaster: Send + Sync {
    fn add_application_listener(&mut self, listener: Arc<dyn ApplicationListener>);
    fn remove_application_listener(&mut self, listener: Arc<dyn ApplicationListener>);
    fn multicast_event(&self, event: Arc<dyn Any + Send + Sync>);
}
