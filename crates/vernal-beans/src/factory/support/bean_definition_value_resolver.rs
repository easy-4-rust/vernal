//! BeanDefinitionValueResolver — Spring 风格 Bean 定义值解析器。

use std::sync::Mutex;
use std::collections::HashMap;

pub struct BeanDefinitionValueResolver {
    resolved_values: Mutex<HashMap<String, String>>,
}

impl BeanDefinitionValueResolver {
    pub fn new() -> Self { Self { resolved_values: Mutex::new(HashMap::new()) } }
    pub fn resolve(&self, key: String, value: String) {
        self.resolved_values.lock().unwrap().insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        self.resolved_values.lock().unwrap().get(key).cloned()
    }
    pub fn resolved_count(&self) -> usize {
        self.resolved_values.lock().unwrap().len()
    }
    pub fn clear(&self) {
        self.resolved_values.lock().unwrap().clear();
    }
}
impl Default for BeanDefinitionValueResolver { fn default() -> Self { Self::new() } }
