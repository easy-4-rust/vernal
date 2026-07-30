//! ManagedList — Spring 风格管理列表。

use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct ManagedList {
    items: Mutex<Vec<Arc<dyn Any + Send + Sync>>>,
    element_type: Option<std::any::TypeId>,
}

impl ManagedList {
    pub fn new() -> Self { Self { items: Mutex::new(Vec::new()), element_type: None } }
    pub fn with_element_type(mut self, type_id: std::any::TypeId) -> Self {
        self.element_type = Some(type_id);
        self
    }
    pub fn add(&self, item: Arc<dyn Any + Send + Sync>) {
        self.items.lock().unwrap().push(item);
    }
    pub fn get(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.items.lock().unwrap().get(index).map(Arc::clone)
    }
    pub fn len(&self) -> usize { self.items.lock().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.items.lock().unwrap().is_empty() }
    pub fn element_type(&self) -> Option<std::any::TypeId> { self.element_type }
}
impl Default for ManagedList { fn default() -> Self { Self::new() } }
