//! 组合属性源。
//!
//! 对标 Spring `org.springframework.core.env.CompositePropertySource`。

use super::PropertySource;
use super::enumerable_property_source::EnumerablePropertySource;

/// 组合属性源。
///
/// 对应 Java: org.springframework.core.env.CompositePropertySource
///
/// Spring 语义：把多个属性源聚合为一个命名的可枚举源——查找时按添加顺序
/// 首个命中返回；`addPropertySource` 追加。
pub struct CompositePropertySource {
    name: String,
    sources: Vec<Box<dyn EnumerablePropertySource>>,
}

impl CompositePropertySource {
    /// 创建命名组合源。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sources: Vec::new(),
        }
    }

    /// 追加属性源（后添加的优先级更高——对标 Spring 语义）。
    pub fn add_property_source(&mut self, source: Box<dyn EnumerablePropertySource>) {
        self.sources.push(source);
    }

    /// 移除指定名称的属性源。
    pub fn remove_property_source(&mut self, name: &str) -> bool {
        let before = self.sources.len();
        self.sources.retain(|source| source.name() != name);
        self.sources.len() != before
    }
}

impl PropertySource for CompositePropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        // 后添加者优先（对标 Spring 组合源查找顺序）
        for source in self.sources.iter().rev() {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
    }
}

impl EnumerablePropertySource for CompositePropertySource {
    fn property_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for source in &self.sources {
            names.extend(source.property_names());
        }
        names.sort();
        names.dedup();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MapPropertySource;

    fn enumerable(name: &str, entries: &[(&str, &str)]) -> Box<dyn EnumerablePropertySource> {
        let mut map = std::collections::HashMap::new();
        for (k, v) in entries {
            map.insert((*k).to_string(), (*v).to_string());
        }
        Box::new(MapPropertySource::new(name, map))
    }

    #[test]
    fn later_added_source_wins() {
        // A 类（合同对齐）：对标 Spring 组合源优先级
        let mut composite = CompositePropertySource::new("composite");
        composite.add_property_source(enumerable("first", &[("key", "first")]));
        composite.add_property_source(enumerable("second", &[("key", "second")]));
        assert_eq!(composite.get_property("key").as_deref(), Some("second"));
        assert_eq!(composite.name(), "composite");
    }

    #[test]
    fn enumerates_unique_names() {
        // B 类（边界行为）：对标 Spring 属性名并集
        let mut composite = CompositePropertySource::new("composite");
        composite.add_property_source(enumerable("a", &[("x", "1"), ("y", "2")]));
        composite.add_property_source(enumerable("b", &[("x", "3")]));
        assert_eq!(
            composite.property_names(),
            vec!["x".to_string(), "y".to_string()]
        );
    }

    #[test]
    fn remove_source_by_name() {
        // A 类（合同对齐）：对标 Spring removePropertySource
        let mut composite = CompositePropertySource::new("composite");
        composite.add_property_source(enumerable("a", &[("x", "1")]));
        assert!(composite.remove_property_source("a"));
        assert!(!composite.remove_property_source("a"));
        assert_eq!(composite.get_property("x"), None);
    }
}
