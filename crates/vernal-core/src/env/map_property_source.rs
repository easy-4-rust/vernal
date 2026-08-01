//! Map 属性源。
//!
//! 对标 Spring `org.springframework.core.env.MapPropertySource`。
//! 基于 `HashMap<String, String>` 的属性源。

use std::collections::HashMap;

use super::property_source::PropertySource;

/// Map 属性源。
///
/// 对应 Java: org.springframework.core.env.MapPropertySource
#[derive(Debug, Clone)]
pub struct MapPropertySource {
    /// 属性源名称
    name: String,
    /// 属性值
    properties: HashMap<String, String>,
}

impl MapPropertySource {
    /// 创建新的 Map 属性源。
    ///
    /// 对应 Java: `MapPropertySource(String, Map<String, Object>)`
    #[must_use]
    pub fn new(name: impl Into<String>, properties: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }

    /// 设置属性值。
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// 移除属性。
    pub fn remove_property(&mut self, key: &str) -> Option<String> {
        self.properties.remove(key)
    }
}

impl super::EnumerablePropertySource for MapPropertySource {
    fn property_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.properties.keys().cloned().collect();
        names.sort();
        names
    }
}

impl PropertySource for MapPropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }
}
