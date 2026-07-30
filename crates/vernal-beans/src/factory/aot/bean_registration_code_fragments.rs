//! bean_registration_code_fragments — 对应 Java 类：org.springframework.beans.factory.aot.BeanRegistrationCodeFragments。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct BeanRegistrationCodeFragments {
    data: Mutex<HashMap<String, String>>,
}

impl BeanRegistrationCodeFragments {
    pub fn new() -> Self { Self { data: Mutex::new(HashMap::new()) } }
    pub fn register(&self, key: String, value: String) {
        self.data.lock().unwrap().insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
    pub fn count(&self) -> usize { self.data.lock().unwrap().len() }
    pub fn cache_size(&self) -> usize { self.count() }
    pub fn clear(&self) { self.data.lock().unwrap().clear(); }
    pub fn contains(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }
    pub fn process(&self, input: String) -> String { input }
}
impl Default for BeanRegistrationCodeFragments { fn default() -> Self { Self::new() } }
