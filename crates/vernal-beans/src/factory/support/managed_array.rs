//! ManagedArray — Spring 风格管理数组。

use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct ManagedArray {
    items: Mutex<Vec<Arc<dyn Any + Send + Sync>>>,
}

impl ManagedArray {
    pub fn new() -> Self { Self { items: Mutex::new(Vec::new()) } }
    pub fn add(&self, item: Arc<dyn Any + Send + Sync>) {
        self.items.lock().unwrap().push(item);
    }
    pub fn get(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.items.lock().unwrap().get(index).map(Arc::clone)
    }
    pub fn len(&self) -> usize { self.items.lock().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.items.lock().unwrap().is_empty() }
}
impl Default for ManagedArray { fn default() -> Self { Self::new() } }
