//! SimpleApplicationEventMulticaster — 简单事件广播器。
use crate::application_event_multicaster::ApplicationEventMulticaster;
use crate::application_listener::ApplicationListener;
use std::any::Any;
use std::sync::{Arc, Mutex};

/// 简单事件广播器。
#[derive(Default)]
pub struct SimpleApplicationEventMulticaster {
    listeners: Arc<Mutex<Vec<Arc<dyn ApplicationListener>>>>,
}
impl SimpleApplicationEventMulticaster {
    pub fn new() -> Self { Self::default() }
}
impl ApplicationEventMulticaster for SimpleApplicationEventMulticaster {
    fn add_application_listener(&mut self, listener: Arc<dyn ApplicationListener>) {
        self.listeners.lock().unwrap().push(listener);
    }
    fn remove_application_listener(&mut self, listener: Arc<dyn ApplicationListener>) {
        self.listeners.lock().unwrap().retain(|l| !Arc::ptr_eq(l, &listener));
    }
    fn multicast_event(&self, event: Arc<dyn Any + Send + Sync>) {
        for listener in self.listeners.lock().unwrap().iter() {
            listener.on_application_event(event.clone());
        }
    }
}
