//! BeanDefinitionValueResolver — Spring 风格 Bean 定义值解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionValueResolver`。
//!
//! 负责将 Bean 定义中的值占位符（如 `${property}`）解析为实际值。
//! 在 Spring 中，此类在 Bean 实例化阶段被调用，将 `BeanDefinition`
//! 中的字符串值替换为容器中的实际对象。

use std::collections::HashMap;
use std::sync::Mutex;

/// Spring 风格 Bean 定义值解析器。
///
/// 对应 Spring 的 `BeanDefinitionValueResolver`。
///
/// 维护一个键值缓存，用于将 Bean 定义中的占位符键
/// 解析为最终的运行时值。
pub struct BeanDefinitionValueResolver {
    resolved_values: Mutex<HashMap<String, String>>,
}

impl BeanDefinitionValueResolver {
    /// 创建新的值解析器。
    pub fn new() -> Self {
        Self {
            resolved_values: Mutex::new(HashMap::new()),
        }
    }

    /// 注册一个已解析的键值对。
    pub fn resolve(&self, key: String, value: String) {
        self.resolved_values.lock().unwrap().insert(key, value);
    }

    /// 获取已解析的值。
    pub fn get(&self, key: &str) -> Option<String> {
        self.resolved_values.lock().unwrap().get(key).cloned()
    }

    /// 已解析的键值对数量。
    pub fn resolved_count(&self) -> usize {
        self.resolved_values.lock().unwrap().len()
    }

    /// 清空已解析的缓存。
    pub fn clear(&self) {
        self.resolved_values.lock().unwrap().clear();
    }

    /// 是否包含指定键。
    pub fn contains_key(&self, key: &str) -> bool {
        self.resolved_values.lock().unwrap().contains_key(key)
    }

    /// 批量注册键值对。
    pub fn resolve_all(&self, entries: impl IntoIterator<Item = (String, String)>) {
        let mut map = self.resolved_values.lock().unwrap();
        for (k, v) in entries {
            map.insert(k, v);
        }
    }
}

impl Default for BeanDefinitionValueResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_and_get() {
        let resolver = BeanDefinitionValueResolver::new();
        resolver.resolve("${db.url}".to_string(), "jdbc:h2:mem".to_string());
        assert_eq!(resolver.get("${db.url}"), Some("jdbc:h2:mem".to_string()));
        assert_eq!(resolver.resolved_count(), 1);
    }

    #[test]
    fn get_missing_returns_none() {
        let resolver = BeanDefinitionValueResolver::new();
        assert_eq!(resolver.get("missing"), None);
        assert!(!resolver.contains_key("missing"));
    }

    #[test]
    fn clear_removes_all() {
        let resolver = BeanDefinitionValueResolver::new();
        resolver.resolve("a".to_string(), "1".to_string());
        resolver.clear();
        assert_eq!(resolver.resolved_count(), 0);
    }

    #[test]
    fn resolve_all_batch() {
        let resolver = BeanDefinitionValueResolver::new();
        resolver.resolve_all(vec![
            ("k1".to_string(), "v1".to_string()),
            ("k2".to_string(), "v2".to_string()),
        ]);
        assert_eq!(resolver.resolved_count(), 2);
        assert!(resolver.contains_key("k1"));
        assert!(resolver.contains_key("k2"));
    }
}
