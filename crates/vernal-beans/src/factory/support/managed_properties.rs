//! ManagedProperties — Spring 风格管理 Properties。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedProperties`。
//!
//! 在 Spring 的 Bean 定义解析过程中，`ManagedProperties` 表示一个
//! `<props>` 元素，其中的键值对都是字符串。
//! 在 Bean 实例化时被解析为 `java.util.Properties`。

use std::collections::HashMap;
use std::sync::Mutex;

/// Spring 风格管理 Properties。
///
/// 对应 Spring 的 `ManagedProperties`。
///
/// 表示 Bean 定义中 `<props>` 元素的值，所有键值对均为字符串。
pub struct ManagedProperties {
    entries: Mutex<HashMap<String, String>>,
}

impl ManagedProperties {
    /// 创建空的管理 Properties。
    pub fn new() -> Self { Self { entries: Mutex::new(HashMap::new()) } }

    /// 设置键值对。
    pub fn set(&self, key: String, value: String) {
        self.entries.lock().unwrap().insert(key, value);
    }

    /// 获取值。
    pub fn get(&self, key: &str) -> Option<String> {
        self.entries.lock().unwrap().get(key).cloned()
    }

    /// 移除键值对。
    pub fn remove(&self, key: &str) -> Option<String> {
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

    /// 清空所有条目。
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// 获取所有值的列表。
    pub fn values(&self) -> Vec<String> {
        self.entries.lock().unwrap().values().cloned().collect()
    }

    /// 获取所有键值对。
    pub fn entries(&self) -> Vec<(String, String)> {
        self.entries.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 获取指定键的值，如果不存在则返回默认值。
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.entries
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// 批量设置多个键值对。
    pub fn set_all(&self, pairs: Vec<(String, String)>) {
        let mut entries = self.entries.lock().unwrap();
        for (k, v) in pairs {
            entries.insert(k, v);
        }
    }

    /// 检查是否包含指定的值。
    pub fn contains_value(&self, value: &str) -> bool {
        self.entries.lock().unwrap().values().any(|v| v == value)
    }
}

impl Default for ManagedProperties { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let props = ManagedProperties::new();
        props.set("db.url".to_string(), "jdbc:h2:mem".to_string());
        assert_eq!(props.get("db.url"), Some("jdbc:h2:mem".to_string()));
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn remove_entry() {
        let props = ManagedProperties::new();
        props.set("key".to_string(), "val".to_string());
        assert!(props.contains_key("key"));
        props.remove("key");
        assert!(!props.contains_key("key"));
        assert!(props.is_empty());
    }

    #[test]
    fn keys_returns_all_keys() {
        let props = ManagedProperties::new();
        props.set("a".to_string(), "1".to_string());
        props.set("b".to_string(), "2".to_string());
        let mut keys = props.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn clear_removes_all() {
        let props = ManagedProperties::new();
        props.set("x".to_string(), "y".to_string());
        props.clear();
        assert!(props.is_empty());
    }

    #[test]
    fn values_returns_all_values() {
        let props = ManagedProperties::new();
        props.set("a".to_string(), "1".to_string());
        props.set("b".to_string(), "2".to_string());
        let mut vals = props.values();
        vals.sort();
        assert_eq!(vals, vec!["1", "2"]);
    }

    #[test]
    fn entries_returns_key_value_pairs() {
        let props = ManagedProperties::new();
        props.set("host".to_string(), "localhost".to_string());
        let entries = props.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], ("host".to_string(), "localhost".to_string()));
    }

    #[test]
    fn get_or_default_returns_existing() {
        let props = ManagedProperties::new();
        props.set("key".to_string(), "value".to_string());
        assert_eq!(props.get_or_default("key", "default"), "value");
        assert_eq!(props.get_or_default("missing", "default"), "default");
    }

    #[test]
    fn set_all_registers_multiple() {
        let props = ManagedProperties::new();
        props.set_all(vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
            ("c".to_string(), "3".to_string()),
        ]);
        assert_eq!(props.len(), 3);
    }

    #[test]
    fn contains_value_checks_values() {
        let props = ManagedProperties::new();
        props.set("key".to_string(), "target".to_string());
        assert!(props.contains_value("target"));
        assert!(!props.contains_value("missing"));
    }
}
