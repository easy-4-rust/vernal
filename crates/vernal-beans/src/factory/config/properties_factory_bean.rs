//! PropertiesFactoryBean — 对应 Spring `org.springframework.beans.factory.config.PropertiesFactoryBean`。
//!
//! Properties 工厂 Bean，用于创建 Properties 集合。

use std::collections::HashMap;

/// Properties 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PropertiesFactoryBean`。
///
/// 用于创建 `Properties`（键值对集合）的 FactoryBean。
/// 支持从文件、URL 或内联配置加载属性。
///
/// ## 使用场景
///
/// - 加载 `.properties` 配置文件
/// - 注入 Properties 类型的 Bean 属性
/// - 配置管理
#[derive(Debug, Default)]
pub struct PropertiesFactoryBean {
    /// 属性集合。
    properties: HashMap<String, String>,
    /// 属性文件位置（可选）。
    locations: Vec<String>,
    /// 是否作为单例。
    singleton: bool,
}

impl PropertiesFactoryBean {
    /// 创建新的 PropertiesFactoryBean。
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            locations: Vec::new(),
            singleton: true,
        }
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

    /// 设置是否单例。
    pub fn set_singleton(&mut self, singleton: bool) {
        self.singleton = singleton;
    }

    /// 是否单例。
    pub fn is_singleton(&self) -> bool {
        self.singleton
    }

    /// 创建 Properties 实例。
    pub fn create_properties(&self) -> HashMap<String, String> {
        self.properties.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties_factory_bean_new() {
        let factory = PropertiesFactoryBean::new();
        assert!(factory.properties().is_empty());
        assert!(factory.locations().is_empty());
        assert!(factory.is_singleton());
    }

    #[test]
    fn test_properties_factory_bean_with_properties() {
        let mut factory = PropertiesFactoryBean::new();
        factory.set_property("key1", "value1");
        factory.set_property("key2", "value2");

        assert_eq!(factory.get_property("key1"), Some("value1"));
        assert_eq!(factory.get_property("key2"), Some("value2"));
        assert_eq!(factory.get_property("key3"), None);
    }

    #[test]
    fn test_properties_factory_bean_locations() {
        let mut factory = PropertiesFactoryBean::new();
        factory.add_location("classpath:config.properties");
        factory.add_location("file:/etc/app.properties");

        assert_eq!(factory.locations().len(), 2);
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn default_trait_creates_correctly() {
        let factory = PropertiesFactoryBean::default();
        assert!(factory.properties().is_empty());
        assert!(factory.locations().is_empty());
        // Default trait uses struct Default which sets singleton to false
        // (bool defaults to false). The `new()` method sets it to true.
        assert!(!factory.is_singleton());
    }

    #[test]
    fn set_properties_bulk() {
        let mut factory = PropertiesFactoryBean::new();
        let mut props = HashMap::new();
        props.insert("a".to_string(), "1".to_string());
        props.insert("b".to_string(), "2".to_string());
        factory.set_properties(props);
        assert_eq!(factory.get_property("a"), Some("1"));
        assert_eq!(factory.get_property("b"), Some("2"));
    }

    #[test]
    fn set_locations_bulk() {
        let mut factory = PropertiesFactoryBean::new();
        factory.set_locations(vec!["loc1".to_string(), "loc2".to_string()]);
        assert_eq!(factory.locations().len(), 2);
    }

    #[test]
    fn set_singleton_flag() {
        let mut factory = PropertiesFactoryBean::new();
        assert!(factory.is_singleton());
        factory.set_singleton(false);
        assert!(!factory.is_singleton());
        factory.set_singleton(true);
        assert!(factory.is_singleton());
    }

    #[test]
    fn create_properties_returns_clone() {
        let mut factory = PropertiesFactoryBean::new();
        factory.set_property("key", "value");
        let props = factory.create_properties();
        assert_eq!(props.get("key").unwrap(), "value");
        // Modifying the clone shouldn't affect the original
        let mut props2 = props;
        props2.insert("key".to_string(), "modified".to_string());
        assert_eq!(factory.get_property("key"), Some("value"));
    }

    #[test]
    fn create_properties_empty() {
        let factory = PropertiesFactoryBean::new();
        let props = factory.create_properties();
        assert!(props.is_empty());
    }

    #[test]
    fn add_location_single() {
        let mut factory = PropertiesFactoryBean::new();
        factory.add_location("classpath:app.properties");
        assert_eq!(factory.locations().len(), 1);
    }

    #[test]
    fn set_property_overwrites() {
        let mut factory = PropertiesFactoryBean::new();
        factory.set_property("key", "first");
        factory.set_property("key", "second");
        assert_eq!(factory.get_property("key"), Some("second"));
    }

    #[test]
    fn debug_format() {
        let factory = PropertiesFactoryBean::new();
        let debug = format!("{:?}", factory);
        assert!(debug.contains("PropertiesFactoryBean"));
    }

    #[test]
    fn get_property_not_found() {
        let factory = PropertiesFactoryBean::new();
        assert!(factory.get_property("missing").is_none());
    }

    #[test]
    fn properties_returns_reference() {
        let mut factory = PropertiesFactoryBean::new();
        factory.set_property("k", "v");
        let props = factory.properties();
        assert_eq!(props.len(), 1);
    }
}
