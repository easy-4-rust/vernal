//! PropertyOverrideConfigurer — 对应 Spring `org.springframework.beans.factory.config.PropertyOverrideConfigurer`。
//!
//! 属性覆盖配置器。

use std::collections::HashMap;

/// 属性覆盖配置器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PropertyOverrideConfigurer`。
///
/// 用于覆盖 Bean 定义中的属性值。
/// 属性键格式：`beanName.propertyName`。
///
/// ## 使用场景
///
/// - 环境特定的属性覆盖
/// - 配置文件中的属性覆盖
/// - 多环境配置管理
#[derive(Debug, Default)]
pub struct PropertyOverrideConfigurer {
    /// 属性集合。
    properties: HashMap<String, String>,
    /// 是否忽略无效键。
    ignore_invalid_keys: bool,
}

impl PropertyOverrideConfigurer {
    /// 创建新的 PropertyOverrideConfigurer。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置属性。
    pub fn set_properties(&mut self, properties: HashMap<String, String>) {
        self.properties = properties;
    }

    /// 添加属性覆盖。
    ///
    /// # 参数
    ///
    /// - `bean_name` — Bean 名称
    /// - `property_name` — 属性名称
    /// - `value` — 属性值
    pub fn add_property_override(
        &mut self,
        bean_name: impl Into<String>,
        property_name: impl Into<String>,
        value: impl Into<String>,
    ) {
        let key = format!("{}.{}", bean_name.into(), property_name.into());
        self.properties.insert(key, value.into());
    }

    /// 获取属性覆盖值。
    pub fn get_property_override(&self, bean_name: &str, property_name: &str) -> Option<&str> {
        let key = format!("{}.{}", bean_name, property_name);
        self.properties.get(&key).map(|s| s.as_str())
    }

    /// 获取所有属性覆盖。
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// 设置是否忽略无效键。
    pub fn set_ignore_invalid_keys(&mut self, ignore: bool) {
        self.ignore_invalid_keys = ignore;
    }

    /// 是否忽略无效键。
    pub fn ignore_invalid_keys(&self) -> bool {
        self.ignore_invalid_keys
    }

    /// 获取指定 Bean 的所有属性覆盖。
    pub fn get_bean_property_overrides(&self, bean_name: &str) -> HashMap<String, String> {
        let prefix = format!("{}.", bean_name);
        self.properties
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(key, value)| {
                let property_name = key[prefix.len()..].to_string();
                (property_name, value.clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_override_configurer_new() {
        let configurer = PropertyOverrideConfigurer::new();
        assert!(configurer.properties().is_empty());
        assert!(!configurer.ignore_invalid_keys());
    }

    #[test]
    fn test_property_override_configurer_add_override() {
        let mut configurer = PropertyOverrideConfigurer::new();
        configurer.add_property_override("dataSource", "url", "jdbc:mysql://localhost/mydb");
        configurer.add_property_override("dataSource", "username", "root");

        assert_eq!(
            configurer.get_property_override("dataSource", "url"),
            Some("jdbc:mysql://localhost/mydb")
        );
        assert_eq!(
            configurer.get_property_override("dataSource", "username"),
            Some("root")
        );
    }

    #[test]
    fn test_property_override_configurer_get_bean_overrides() {
        let mut configurer = PropertyOverrideConfigurer::new();
        configurer.add_property_override("dataSource", "url", "jdbc:mysql://localhost/mydb");
        configurer.add_property_override("dataSource", "username", "root");
        configurer.add_property_override("httpClient", "timeout", "5000");

        let overrides = configurer.get_bean_property_overrides("dataSource");
        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides.get("url"), Some(&"jdbc:mysql://localhost/mydb".to_string()));
        assert_eq!(overrides.get("username"), Some(&"root".to_string()));
    }
}
