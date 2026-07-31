//! PropertyResourceConfigurer — 对应 Spring `org.springframework.beans.factory.config.PropertyResourceConfigurer`。
//!
//! 属性资源配置器基类。

use std::collections::HashMap;

/// 属性资源配置器基类。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PropertyResourceConfigurer`。
///
/// 用于从外部资源（文件、URL 等）加载属性，并将其应用于 Bean 定义。
///
/// ## 使用场景
///
/// - 加载外部配置文件
/// - 配置属性覆盖
/// - 环境特定配置
#[derive(Debug, Default)]
pub struct PropertyResourceConfigurer {
    /// 属性集合。
    properties: HashMap<String, String>,
    /// 属性文件位置。
    locations: Vec<String>,
    /// 文件编码。
    file_encoding: Option<String>,
}

impl PropertyResourceConfigurer {
    /// 创建新的 PropertyResourceConfigurer。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置属性。
    pub fn set_properties(&mut self, properties: HashMap<String, String>) {
        self.properties = properties;
    }

    /// 添加属性。
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// 获取属性值。
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    /// 获取所有属性。
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 设置属性文件位置。
    pub fn set_locations(&mut self, locations: Vec<String>) {
        self.locations = locations;
    }

    /// 添加属性文件位置。
    pub fn add_location(&mut self, location: impl Into<String>) {
        self.locations.push(location.into());
    }

    /// 获取属性文件位置。
    pub fn locations(&self) -> &[String] {
        &self.locations
    }

    /// 设置文件编码。
    pub fn set_file_encoding(&mut self, encoding: impl Into<String>) {
        self.file_encoding = Some(encoding.into());
    }

    /// 获取文件编码。
    pub fn file_encoding(&self) -> Option<&str> {
        self.file_encoding.as_deref()
    }

    /// 合并属性。
    pub fn merge_properties(&mut self, other: &HashMap<String, String>) {
        for (key, value) in other {
            self.properties.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_resource_configurer_new() {
        let configurer = PropertyResourceConfigurer::new();
        assert!(configurer.properties().is_empty());
        assert!(configurer.locations().is_empty());
        assert!(configurer.file_encoding().is_none());
    }

    #[test]
    fn test_property_resource_configurer_properties() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_property("key1", "value1");
        configurer.set_property("key2", "value2");

        assert_eq!(configurer.get_property("key1"), Some("value1"));
        assert_eq!(configurer.get_property("key2"), Some("value2"));
        assert_eq!(configurer.get_property("key3"), None);
    }

    #[test]
    fn test_property_resource_configurer_merge() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_property("key1", "value1");

        let mut other = HashMap::new();
        other.insert("key2".to_string(), "value2".to_string());
        other.insert("key1".to_string(), "new_value1".to_string());

        configurer.merge_properties(&other);

        assert_eq!(configurer.get_property("key1"), Some("new_value1"));
        assert_eq!(configurer.get_property("key2"), Some("value2"));
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn default_trait_creates_empty() {
        let configurer = PropertyResourceConfigurer::default();
        assert!(configurer.properties().is_empty());
        assert!(configurer.locations().is_empty());
        assert!(configurer.file_encoding().is_none());
    }

    #[test]
    fn set_properties_bulk() {
        let mut configurer = PropertyResourceConfigurer::new();
        let mut props = HashMap::new();
        props.insert("a".to_string(), "1".to_string());
        props.insert("b".to_string(), "2".to_string());
        configurer.set_properties(props);
        assert_eq!(configurer.get_property("a"), Some("1"));
        assert_eq!(configurer.get_property("b"), Some("2"));
    }

    #[test]
    fn set_locations_bulk() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_locations(vec![
            "file:/a.properties".to_string(),
            "file:/b.properties".to_string(),
        ]);
        assert_eq!(configurer.locations().len(), 2);
    }

    #[test]
    fn add_location_single() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.add_location("classpath:app.properties");
        assert_eq!(configurer.locations().len(), 1);
        assert_eq!(configurer.locations()[0], "classpath:app.properties");
    }

    #[test]
    fn set_file_encoding() {
        let mut configurer = PropertyResourceConfigurer::new();
        assert!(configurer.file_encoding().is_none());
        configurer.set_file_encoding("UTF-8");
        assert_eq!(configurer.file_encoding(), Some("UTF-8"));
    }

    #[test]
    fn merge_empty_properties() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_property("key", "value");
        let other = HashMap::new();
        configurer.merge_properties(&other);
        assert_eq!(configurer.get_property("key"), Some("value"));
    }

    #[test]
    fn set_property_overwrites() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_property("key", "first");
        assert_eq!(configurer.get_property("key"), Some("first"));
        configurer.set_property("key", "second");
        assert_eq!(configurer.get_property("key"), Some("second"));
    }

    #[test]
    fn debug_format() {
        let configurer = PropertyResourceConfigurer::new();
        let debug = format!("{:?}", configurer);
        assert!(debug.contains("PropertyResourceConfigurer"));
    }

    #[test]
    fn properties_returns_reference() {
        let mut configurer = PropertyResourceConfigurer::new();
        configurer.set_property("k", "v");
        let props = configurer.properties();
        assert_eq!(props.len(), 1);
        assert_eq!(props.get("k").unwrap(), "v");
    }
}
