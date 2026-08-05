//! 可变属性源集合。
//!
//! 对标 Spring `org.springframework.core.env.MutablePropertySources`。

use super::PropertySources;
use super::property_source::PropertySource;

/// 可变属性源集合。
///
/// 对应 Java: org.springframework.core.env.MutablePropertySources
///
/// Spring 语义：按优先级管理属性源（`addFirst` 最高优先级 / `addLast` 最低
/// 优先级 / `remove` / `contains`），查找时首个命中即返回。
pub struct MutablePropertySources {
    sources: Vec<Box<dyn PropertySource>>,
}

impl MutablePropertySources {
    /// 创建空的属性源集合。
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// 在最高优先级位置加入属性源。
    ///
    /// 对应 Java: `MutablePropertySources#addFirst(PropertySource)`
    pub fn add_first(&mut self, source: Box<dyn PropertySource>) {
        self.sources.insert(0, source);
    }

    /// 在最低优先级位置加入属性源。
    ///
    /// 对应 Java: `MutablePropertySources#addLast(PropertySource)`
    pub fn add_last(&mut self, source: Box<dyn PropertySource>) {
        self.sources.push(source);
    }

    /// 按名称移除属性源。
    ///
    /// 对应 Java: `MutablePropertySources#remove(String)`
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.sources.len();
        self.sources.retain(|source| source.name() != name);
        self.sources.len() != before
    }

    /// 判断是否包含指定名称的属性源。
    ///
    /// 对应 Java: `MutablePropertySources#contains(String)`
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.sources.iter().any(|source| source.name() == name)
    }

    /// 按优先级顺序查找属性值（首个命中返回）。
    #[must_use]
    pub fn get_property(&self, key: &str) -> Option<String> {
        for source in &self.sources {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
    }

    /// 检查属性是否存在。
    #[must_use]
    pub fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }
}

impl Default for MutablePropertySources {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertySources for MutablePropertySources {
    fn sources(&self) -> Vec<&dyn PropertySource> {
        self.sources.iter().map(Box::as_ref).collect()
    }

    fn len(&self) -> usize {
        self.sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MapPropertySource;
    use std::collections::HashMap;

    fn source(name: &str, key: &str, value: &str) -> Box<dyn PropertySource> {
        let mut map = HashMap::new();
        map.insert(key.to_string(), value.to_string());
        Box::new(MapPropertySource::new(name, map))
    }

    #[test]
    fn first_match_wins_by_priority() {
        // A 类（合同对齐）：对标 Spring addFirst/addLast 优先级
        let mut sources = MutablePropertySources::new();
        sources.add_last(source("low", "key", "low"));
        sources.add_first(source("high", "key", "high"));
        assert_eq!(sources.get_property("key").as_deref(), Some("high"));
    }

    #[test]
    fn remove_and_contains() {
        // A 类（合同对齐）：对标 Spring remove/contains
        let mut sources = MutablePropertySources::new();
        sources.add_last(source("a", "k", "v"));
        assert!(sources.contains("a"));
        assert!(sources.remove("a"));
        assert!(!sources.contains("a"));
        assert!(!sources.remove("a"));
    }

    #[test]
    fn missing_property_returns_none() {
        // B 类（边界行为）
        let sources = MutablePropertySources::new();
        assert_eq!(sources.get_property("missing"), None);
        assert!(!sources.contains_property("missing"));
    }

    #[test]
    fn iteration_order_matches_priority() {
        // D 类（重构安全）：PropertySources 契约的迭代顺序
        let mut sources = MutablePropertySources::new();
        sources.add_last(source("low", "a", "1"));
        sources.add_first(source("high", "b", "2"));
        let names: Vec<&str> = sources.sources().iter().map(|s| s.name()).collect();
        assert_eq!(names, vec!["high", "low"]);
        assert_eq!(sources.len(), 2);
    }
}
