use crate::application_listener::ApplicationListener;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AbstractApplicationEventMulticaster {
    listeners: RwLock<BTreeMap<u64, Arc<dyn ApplicationListener>>>,
    next_id: AtomicU64,
}

impl AbstractApplicationEventMulticaster {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_listener(&self, listener: Arc<dyn ApplicationListener>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        self.listeners
            .write()
            .expect("listener lock poisoned")
            .insert(id, listener);
        id
    }
    pub fn remove_listener(&self, id: u64) -> bool {
        self.listeners
            .write()
            .expect("listener lock poisoned")
            .remove(&id)
            .is_some()
    }
    pub fn clear_listeners(&self) {
        self.listeners
            .write()
            .expect("listener lock poisoned")
            .clear();
    }
    pub fn listeners(&self) -> Vec<Arc<dyn ApplicationListener>> {
        self.listeners
            .read()
            .expect("listener lock poisoned")
            .values()
            .cloned()
            .collect()
    }
    pub fn listener_count(&self) -> usize {
        self.listeners.read().expect("listener lock poisoned").len()
    }
}
