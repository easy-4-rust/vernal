//! YamlPropertiesFactoryBean — 对应 Spring `org.springframework.beans.factory.config.YamlPropertiesFactoryBean`。
//!
//! YAML Properties 工厂 Bean。

use std::collections::HashMap;

/// YAML Properties 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.YamlPropertiesFactoryBean`。
///
/// 用于从 YAML 文件创建 Properties 集合。
///
/// ## 使用场景
///
/// - 加载 YAML 配置文件
/// - 将 YAML 配置转换为 Properties
/// - 多环境 YAML 配置
#[derive(Debug, Default)]
pub struct YamlPropertiesFactoryBean {
    /// 资源位置。
    resources: Vec<String>,
    /// 属性集合。
    properties: HashMap<String, String>,
}

impl YamlPropertiesFactoryBean {
    /// 创建新的 YamlPropertiesFactoryBean。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置资源位置。
    pub fn set_resources(&mut self, resources: Vec<String>) {
        self.resources = resources;
    }

    /// 添加资源位置。
    pub fn add_resource(&mut self, resource: impl Into<String>) {
        self.resources.push(resource.into());
    }

    /// 获取资源位置。
    pub fn resources(&self) -> &[String] {
        &self.resources
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

    /// 从 YAML 文本加载属性。
    pub fn load_from_yaml(&mut self, yaml_text: &str) {
        let mut current_prefix = String::new();

        for line in yaml_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                if value.is_empty() {
                    // 嵌套对象
                    if current_prefix.is_empty() {
                        current_prefix = key.to_string();
                    } else {
                        current_prefix = format!("{}.{}", current_prefix, key);
                    }
                } else {
                    let full_key = if current_prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", current_prefix, key)
                    };
                    self.properties.insert(full_key, value.to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_properties_factory_bean_new() {
        let factory = YamlPropertiesFactoryBean::new();
        assert!(factory.resources().is_empty());
        assert!(factory.properties().is_empty());
    }

    #[test]
    fn test_yaml_properties_factory_bean_load() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "app:\n  name: MyApp\n  version: 1.0.0";
        factory.load_from_yaml(yaml);

        assert_eq!(factory.get_property("app.name"), Some("MyApp"));
        assert_eq!(factory.get_property("app.version"), Some("1.0.0"));
    }

    #[test]
    fn test_yaml_properties_factory_bean_set_property() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.set_property("key1", "value1");
        factory.set_property("key2", "value2");

        assert_eq!(factory.get_property("key1"), Some("value1"));
        assert_eq!(factory.get_property("key2"), Some("value2"));
    }

    #[test]
    fn test_set_resources() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.set_resources(vec!["file1.yaml".to_string(), "file2.yaml".to_string()]);
        assert_eq!(factory.resources().len(), 2);
        assert_eq!(factory.resources()[0], "file1.yaml");
    }

    #[test]
    fn test_add_resource() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.add_resource("file1.yaml");
        factory.add_resource("file2.yaml");
        assert_eq!(factory.resources().len(), 2);
    }

    #[test]
    fn test_set_properties_bulk() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let mut props = std::collections::HashMap::new();
        props.insert("k1".to_string(), "v1".to_string());
        props.insert("k2".to_string(), "v2".to_string());
        factory.set_properties(props);
        assert_eq!(factory.get_property("k1"), Some("v1"));
        assert_eq!(factory.get_property("k2"), Some("v2"));
    }

    #[test]
    fn test_get_property_not_found() {
        let factory = YamlPropertiesFactoryBean::new();
        assert!(factory.get_property("missing").is_none());
    }

    #[test]
    fn test_load_from_yaml_flat() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "name: MyApp\nversion: 1.0\nport: 8080";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("name"), Some("MyApp"));
        assert_eq!(factory.get_property("version"), Some("1.0"));
        assert_eq!(factory.get_property("port"), Some("8080"));
    }

    #[test]
    fn test_load_from_yaml_nested() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "server:\n  host: localhost\n  port: 8080";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("server.host"), Some("localhost"));
        assert_eq!(factory.get_property("server.port"), Some("8080"));
    }

    #[test]
    fn test_load_from_yaml_comments_skipped() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "# comment\nname: MyApp\n# another comment\nversion: 1.0";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("name"), Some("MyApp"));
        assert_eq!(factory.get_property("version"), Some("1.0"));
    }

    #[test]
    fn test_load_from_yaml_empty_lines_skipped() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "name: MyApp\n\n\nversion: 1.0";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("name"), Some("MyApp"));
        assert_eq!(factory.get_property("version"), Some("1.0"));
    }

    #[test]
    fn test_load_from_yaml_empty() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.load_from_yaml("");
        assert!(factory.properties().is_empty());
    }

    #[test]
    fn test_properties_returns_all() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.set_property("k1", "v1");
        factory.set_property("k2", "v2");
        let props = factory.properties();
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_default_trait() {
        let factory = YamlPropertiesFactoryBean::default();
        assert!(factory.resources().is_empty());
        assert!(factory.properties().is_empty());
    }

    #[test]
    fn test_load_from_yaml_deeply_nested() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "app:\n  database:\n    host: localhost\n    port: 5432";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("app.database.host"), Some("localhost"));
        assert_eq!(factory.get_property("app.database.port"), Some("5432"));
    }

    #[test]
    fn test_load_from_yaml_only_comments() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "# comment1\n# comment2\n# comment3";
        factory.load_from_yaml(yaml);
        assert!(factory.properties().is_empty());
    }

    #[test]
    fn test_load_from_yaml_mixed_content() {
        let mut factory = YamlPropertiesFactoryBean::new();
        let yaml = "# Config\nname: MyApp\n\n# Database\nserver:\n  host: localhost\n  port: 5432";
        factory.load_from_yaml(yaml);
        assert_eq!(factory.get_property("name"), Some("MyApp"));
        assert_eq!(factory.get_property("server.host"), Some("localhost"));
        assert_eq!(factory.get_property("server.port"), Some("5432"));
    }

    #[test]
    fn test_set_property_overwrites() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.set_property("key", "value1");
        assert_eq!(factory.get_property("key"), Some("value1"));
        factory.set_property("key", "value2");
        assert_eq!(factory.get_property("key"), Some("value2"));
    }

    #[test]
    fn test_properties_count() {
        let mut factory = YamlPropertiesFactoryBean::new();
        factory.set_property("a", "1");
        factory.set_property("b", "2");
        factory.set_property("c", "3");
        assert_eq!(factory.properties().len(), 3);
    }
}
