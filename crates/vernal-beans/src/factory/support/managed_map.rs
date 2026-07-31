//! ManagedMap — Spring 风格管理映射。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedMap`。
//!
//! 在 Spring 的 Bean 定义解析过程中，`ManagedMap` 表示一个
//! `<map>` 元素。键和值可以是未解析的 Bean 引用，
//! 在 Bean 实例化时被解析为真实对象。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spring 风格管理映射。
///
/// 对应 Spring 的 `ManagedMap`。
///
/// 表示 Bean 定义中 `<map>` 元素的值。
pub struct ManagedMap {
    entries: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl ManagedMap {
    /// 创建空的管理映射。
    pub fn new() -> Self { Self { entries: Mutex::new(HashMap::new()) } }

    /// 插入键值对。
    pub fn put(&self, key: String, value: Arc<dyn Any + Send + Sync>) {
        self.entries.lock().unwrap().insert(key, value);
    }

    /// 获取值。
    pub fn get(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries.lock().unwrap().get(key).map(Arc::clone)
    }

    /// 移除键值对。
    pub fn remove(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries.lock().unwrap().remove(key)
    }

    /// 键值对数量。
    pub fn len(&self) -> usize { self.entries.lock().unwrap().len() }

    /// 是否为空。
    pub fn is_empty(&self) -> bool { self.entries.lock().unwrap().is_empty() }

    /// 是否包含指定键。
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.lock().unwrap().contains_key(key)
    }

    /// 所有键的列表。
    pub fn keys(&self) -> Vec<String> {
        self.entries.lock().unwrap().keys().cloned().collect()
    }

    /// 清空映射。
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// 获取所有值的列表。
    pub fn values(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.entries.lock().unwrap().values().cloned().collect()
    }

    /// 获取所有键值对。
    pub fn entries(&self) -> Vec<(String, Arc<dyn Any + Send + Sync>)> {
        self.entries.lock().unwrap().iter().map(|(k, v)| (k.clone(), Arc::clone(v))).collect()
    }

    /// 获取指定键的值，如果不存在则插入默认值。
    pub fn get_or_insert(&self, key: String, default: Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> {
        let mut entries = self.entries.lock().unwrap();
        if let Some(val) = entries.get(&key) {
            Arc::clone(val)
        } else {
            entries.insert(key, Arc::clone(&default));
            default
        }
    }

    /// 合并另一个 ManagedMap 的所有条目。
    pub fn merge(&self, other: &ManagedMap) {
        let other_entries = other.entries.lock().unwrap().clone();
        let mut entries = self.entries.lock().unwrap();
        for (k, v) in other_entries {
            entries.insert(k, v);
        }
    }
}

impl Default for ManagedMap { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_get() {
        let map = ManagedMap::new();
        map.put("key1".to_string(), Arc::new(100_i32));
        assert_eq!(*map.get("key1").unwrap().downcast_ref::<i32>().unwrap(), 100);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_entry() {
        let map = ManagedMap::new();
        map.put("k".to_string(), Arc::new("v".to_string()));
        assert!(map.contains_key("k"));
        map.remove("k");
        assert!(!map.contains_key("k"));
        assert!(map.is_empty());
    }

    #[test]
    fn keys_returns_all_keys() {
        let map = ManagedMap::new();
        map.put("a".to_string(), Arc::new(1));
        map.put("b".to_string(), Arc::new(2));
        let mut keys = map.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn clear_removes_all() {
        let map = ManagedMap::new();
        map.put("x".to_string(), Arc::new(1));
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn values_returns_all_values() {
        let map = ManagedMap::new();
        map.put("a".to_string(), Arc::new(1_i32));
        map.put("b".to_string(), Arc::new(2_i32));
        assert_eq!(map.values().len(), 2);
    }

    #[test]
    fn entries_returns_key_value_pairs() {
        let map = ManagedMap::new();
        map.put("x".to_string(), Arc::new(10_i32));
        let entries = map.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "x");
        assert_eq!(*entries[0].1.downcast_ref::<i32>().unwrap(), 10);
    }

    #[test]
    fn get_or_insert_creates_default() {
        let map = ManagedMap::new();
        let val = map.get_or_insert("key".to_string(), Arc::new(42_i32));
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);

        // 第二次获取应该返回已存在的值
        let val2 = map.get_or_insert("key".to_string(), Arc::new(99_i32));
        assert_eq!(*val2.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn merge_combines_maps() {
        let map1 = ManagedMap::new();
        map1.put("a".to_string(), Arc::new(1_i32));

        let map2 = ManagedMap::new();
        map2.put("b".to_string(), Arc::new(2_i32));
        map2.put("c".to_string(), Arc::new(3_i32));

        map1.merge(&map2);
        assert_eq!(map1.len(), 3);
        assert!(map1.contains_key("a"));
        assert!(map1.contains_key("b"));
        assert!(map1.contains_key("c"));
    }
}
