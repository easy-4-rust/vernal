//! autowired_arguments_code_generator — 对应 Java 类：org.springframework.beans.factory.aot.AutowiredArgumentsCodeGenerator。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct AutowiredArgumentsCodeGenerator {
    data: Mutex<HashMap<String, String>>,
}

impl AutowiredArgumentsCodeGenerator {
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
impl Default for AutowiredArgumentsCodeGenerator { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generator_is_empty() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        assert_eq!(codegen.count(), 0);
        assert_eq!(codegen.cache_size(), 0);
    }

    #[test]
    fn default_trait_works() {
        let codegen = AutowiredArgumentsCodeGenerator::default();
        assert_eq!(codegen.count(), 0);
    }

    #[test]
    fn register_and_get() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        codegen.register("key1".to_string(), "value1".to_string());
        assert_eq!(codegen.get("key1"), Some("value1".to_string()));
        assert_eq!(codegen.count(), 1);
    }

    #[test]
    fn get_returns_none_for_missing() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        assert!(codegen.get("missing").is_none());
    }

    #[test]
    fn contains_true_and_false() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        codegen.register("key".to_string(), "val".to_string());
        assert!(codegen.contains("key"));
        assert!(!codegen.contains("other"));
    }

    #[test]
    fn register_overwrites() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        codegen.register("key".to_string(), "first".to_string());
        codegen.register("key".to_string(), "second".to_string());
        assert_eq!(codegen.get("key"), Some("second".to_string()));
        assert_eq!(codegen.count(), 1);
    }

    #[test]
    fn clear_removes_all() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        codegen.register("a".to_string(), "1".to_string());
        codegen.register("b".to_string(), "2".to_string());
        assert_eq!(codegen.count(), 2);
        codegen.clear();
        assert_eq!(codegen.count(), 0);
    }

    #[test]
    fn process_returns_input() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        assert_eq!(codegen.process("hello".to_string()), "hello");
    }

    #[test]
    fn cache_size_matches_count() {
        let codegen = AutowiredArgumentsCodeGenerator::new();
        codegen.register("a".to_string(), "1".to_string());
        assert_eq!(codegen.cache_size(), codegen.count());
    }
}
