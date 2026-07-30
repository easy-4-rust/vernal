//! Cache — 缓存系统。
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 缓存 trait。
pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> Option<Arc<dyn std::any::Any + Send + Sync>>;
    fn put(&self, key: &str, value: Arc<dyn std::any::Any + Send + Sync>);
    fn evict(&self, key: &str);
    fn clear(&self);
}

/// 简单内存缓存。
#[derive(Default)]
pub struct SimpleCache {
    store: Arc<Mutex<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>>,
}
impl SimpleCache {
    pub fn new() -> Self { Self::default() }
}
impl Cache for SimpleCache {
    fn get(&self, key: &str) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.store.lock().unwrap().get(key).cloned()
    }
    fn put(&self, key: &str, value: Arc<dyn std::any::Any + Send + Sync>) {
        self.store.lock().unwrap().insert(key.to_string(), value);
    }
    fn evict(&self, key: &str) { self.store.lock().unwrap().remove(key); }
    fn clear(&self) { self.store.lock().unwrap().clear(); }
}

impl std::fmt::Debug for SimpleCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCache").finish()
    }
}
