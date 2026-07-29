use crate::{
    abstract_application_event_multicaster::AbstractApplicationEventMulticaster,
    application_event::ApplicationEvent,
    application_event_multicaster::ApplicationEventMulticaster,
    application_listener::ApplicationListener,
};
use std::sync::Arc;

#[derive(Default)]
pub struct SimpleApplicationEventMulticaster {
    base: AbstractApplicationEventMulticaster,
}
impl SimpleApplicationEventMulticaster {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn listener_count(&self) -> usize {
        self.base.listener_count()
    }
}
impl ApplicationEventMulticaster for SimpleApplicationEventMulticaster {
    fn multicast_event(&self, event: &dyn ApplicationEvent) {
        for listener in self.base.listeners() {
            listener.on_application_event(event);
        }
    }
    fn add_application_listener(&self, listener: Arc<dyn ApplicationListener>) -> u64 {
        self.base.add_listener(listener)
    }
    fn remove_application_listener(&self, id: u64) -> bool {
        self.base.remove_listener(id)
    }
    fn remove_all_listeners(&self) {
        self.base.clear_listeners();
    }
}
