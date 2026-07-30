//! ManagedSet — Spring 风格管理集合。

use std::any::Any;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::Arc;

pub struct ManagedSet {
    items: Mutex<HashSet<String>>,
}

impl ManagedSet {
    pub fn new() -> Self { Self { items: Mutex::new(HashSet::new()) } }
    pub fn add(&self, key: &str) -> bool {
        self.items.lock().unwrap().insert(key.to_string())
    }
    pub fn contains(&self, key: &str) -> bool {
        self.items.lock().unwrap().contains(key)
    }
    pub fn len(&self) -> usize { self.items.lock().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.items.lock().unwrap().is_empty() }
    pub fn remove(&self, key: &str) -> bool {
        self.items.lock().unwrap().remove(key)
    }
}
impl Default for ManagedSet { fn default() -> Self { Self::new() } }
