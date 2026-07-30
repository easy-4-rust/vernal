//! ManagedProperties — Spring 风格管理 Properties。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct ManagedProperties {
    entries: Mutex<HashMap<String, String>>,
}

impl ManagedProperties {
    pub fn new() -> Self { Self { entries: Mutex::new(HashMap::new()) } }
    pub fn set(&self, key: String, value: String) {
        self.entries.lock().unwrap().insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        self.entries.lock().unwrap().get(key).cloned()
    }
    pub fn remove(&self, key: &str) -> Option<String> {
        self.entries.lock().unwrap().remove(key)
    }
    pub fn len(&self) -> usize { self.entries.lock().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.entries.lock().unwrap().is_empty() }
}
impl Default for ManagedProperties { fn default() -> Self { Self::new() } }
