//! ManagedMap — Spring 风格管理映射。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ManagedMap {
    entries: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl ManagedMap {
    pub fn new() -> Self { Self { entries: Mutex::new(HashMap::new()) } }
    pub fn put(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        self.entries.lock().unwrap().insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries.lock().unwrap().get(key).map(Arc::clone)
    }
    pub fn remove(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries.lock().unwrap().remove(key)
    }
    pub fn len(&self) -> usize { self.entries.lock().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.entries.lock().unwrap().is_empty() }
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.lock().unwrap().contains_key(key)
    }
}
impl Default for ManagedMap { fn default() -> Self { Self::new() } }
