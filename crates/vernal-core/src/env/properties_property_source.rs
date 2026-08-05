//! 属性表属性源。
//!
//! 对标 Spring `org.springframework.core.env.PropertiesPropertySource`。

use std::collections::HashMap;

use super::PropertySource;
use super::enumerable_property_source::EnumerablePropertySource;

/// 属性表属性源。
///
/// 对应 Java: org.springframework.core.env.PropertiesPropertySource
///
/// Spring 语义：`MapPropertySource` 的属性表形态（`java.util.Properties`
/// 由 `HashMap<String, String>` 表达）。
pub struct PropertiesPropertySource {
    name: String,
    properties: HashMap<String, String>,
}

impl PropertiesPropertySource {
    /// 从属性表创建命名源。
    #[must_use]
    pub fn new(name: impl Into<String>, properties: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }

    /// 返回底层属性表引用。
    #[must_use]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }
}

impl PropertySource for PropertiesPropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }

    fn contains_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

impl EnumerablePropertySource for PropertiesPropertySource {
    fn property_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.properties.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_properties() {
        // A 类（合同对齐）：对标 Spring 属性读取
        let mut map = HashMap::new();
        map.insert("a".to_string(), "1".to_string());
        let source = PropertiesPropertySource::new("props", map);
        assert_eq!(source.name(), "props");
        assert_eq!(source.get_property("a").as_deref(), Some("1"));
        assert!(source.contains_property("a"));
        assert!(!source.contains_property("b"));
    }

    #[test]
    fn enumerates_sorted_names() {
        // B 类（边界行为）
        let mut map = HashMap::new();
        map.insert("b".to_string(), "2".to_string());
        map.insert("a".to_string(), "1".to_string());
        let source = PropertiesPropertySource::new("props", map);
        assert_eq!(
            source.property_names(),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
