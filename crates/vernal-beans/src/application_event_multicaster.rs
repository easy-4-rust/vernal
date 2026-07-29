use crate::{application_event::ApplicationEvent, application_listener::ApplicationListener};
use std::sync::Arc;

pub trait ApplicationEventMulticaster: Send + Sync {
    fn multicast_event(&self, event: &dyn ApplicationEvent);
    fn add_application_listener(&self, listener: Arc<dyn ApplicationListener>) -> u64;
    fn remove_application_listener(&self, listener_id: u64) -> bool;
    fn remove_all_listeners(&self);
}
