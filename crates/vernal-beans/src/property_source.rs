//! PropertySource — Spring 风格的属性源。
//!
//! 对应 Java 类：`org.springframework.core.env.PropertySource`。
//!
//! 表示一个具名的属性来源，内部存储键值对。

use std::collections::HashMap;

/// Spring 风格的属性源。
///
/// 对应 Spring 的 `PropertySource<T>`（此处 `T` 固定为 `HashMap<String, String>`，
/// 对应 `MapPropertySource`）。
///
/// 每个属性源有一个名称和一个底层键值存储。
#[derive(Debug, Clone)]
pub struct PropertySource {
    /// 属性源名称。
    name: String,
    /// 底层键值存储。
    source: HashMap<String, String>,
}

impl PropertySource {
    /// 创建空的属性源。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: HashMap::new(),
        }
    }

    /// 从已有映射创建属性源。
    pub fn with_source(name: impl Into<String>, source: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            source,
        }
    }

    /// 从键值对迭代器创建属性源。
    pub fn from_pairs<I, K, V>(name: impl Into<String>, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut source = HashMap::new();
        for (k, v) in pairs {
            source.insert(k.into(), v.into());
        }
        Self {
            name: name.into(),
            source,
        }
    }

    /// 获取属性源名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 是否包含指定属性。
    pub fn contains_property(&self, name: &str) -> bool {
        self.source.contains_key(name)
    }

    /// 获取指定属性值。
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.source.get(name).map(String::as_str)
    }

    /// 获取所有属性名。
    pub fn property_names(&self) -> Vec<&str> {
        self.source.keys().map(String::as_str).collect()
    }

    /// 获取底层存储的引用。
    pub fn source(&self) -> &HashMap<String, String> {
        &self.source
    }

    /// 获取底层存储的可变引用。
    pub fn source_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.source
    }

    /// 设置一个属性。
    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.source.insert(name.into(), value.into());
    }

    /// 移除一个属性。
    pub fn remove_property(&mut self, name: &str) -> Option<String> {
        self.source.remove(name)
    }

    /// 属性数量。
    pub fn len(&self) -> usize {
        self.source.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_source() {
        let mut ps = PropertySource::from_pairs("app", [("name", "vernal"), ("version", "1")]);
        assert_eq!(ps.name(), "app");
        assert!(ps.contains_property("name"));
        assert_eq!(ps.get_property("version"), Some("1"));
        ps.set_property("version", "2");
        assert_eq!(ps.get_property("version"), Some("2"));
        assert_eq!(ps.len(), 2);
    }
}
