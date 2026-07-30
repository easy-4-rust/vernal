//! BeanDefinitionReaderUtils — Bean 定义读取器工具。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct BeanDefinitionReaderUtils {
    name_counters: Mutex<HashMap<String, u32>>,
}

impl BeanDefinitionReaderUtils {
    pub fn new() -> Self { Self { name_counters: Mutex::new(HashMap::new()) } }
    pub fn generate_bean_name(&self, prefix: &str) -> String {
        let mut counters = self.name_counters.lock().unwrap();
        let count = counters.entry(prefix.to_string()).or_insert(0);
        *count += 1;
        format!("{}#{}", prefix, count)
    }
    pub fn reset_counter(&self, prefix: &str) {
        self.name_counters.lock().unwrap().remove(prefix);
    }
}
impl Default for BeanDefinitionReaderUtils { fn default() -> Self { Self::new() } }
