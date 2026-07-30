//! BeanFactoryAnnotationUtils — Bean 工厂注解工具。

use std::collections::HashSet;
use std::sync::Mutex;

pub struct BeanFactoryAnnotationUtils {
    processed_annotations: Mutex<HashSet<String>>,
}

impl BeanFactoryAnnotationUtils {
    pub fn new() -> Self {
        Self { processed_annotations: Mutex::new(HashSet::new()) }
    }
    pub fn mark_processed(&self, name: String) {
        self.processed_annotations.lock().unwrap().insert(name);
    }
    pub fn is_processed(&self, name: &str) -> bool {
        self.processed_annotations.lock().unwrap().contains(name)
    }
    pub fn processed_count(&self) -> usize {
        self.processed_annotations.lock().unwrap().len()
    }
}
impl Default for BeanFactoryAnnotationUtils { fn default() -> Self { Self::new() } }
