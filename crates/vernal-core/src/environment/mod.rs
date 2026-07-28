//! 环境抽象模块。
//!
//! 对标 Spring `org.springframework.core.env.Environment` 和 `PropertySource`。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `Environment` interface | `Environment` trait |
//! | `PropertySource` abstract class | `PropertySource` trait |
//! | `PropertySources` | `PropertySources` struct |
//! | `MapPropertySource` | `MapPropertySource` struct |

use std::collections::HashMap;

/// 属性源 trait。
///
/// 对标 Spring `org.springframework.core.env.PropertySource`。
pub trait PropertySource: Send + Sync {
    /// 属性源名称。
    fn name(&self) -> &str;

    /// 获取属性值。
    fn get_property(&self, key: &str) -> Option<String>;

    /// 检查属性是否存在。
    fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }
}

/// Map 属性源。
///
/// 对标 Spring `org.springframework.core.env.MapPropertySource`。
#[derive(Debug, Clone)]
pub struct MapPropertySource {
    /// 属性源名称
    name: String,
    /// 属性值
    properties: HashMap<String, String>,
}

impl MapPropertySource {
    /// 创建新的 Map 属性源。
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

impl PropertySource for MapPropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }
}

/// 系统环境变量属性源。
///
/// 对标 Spring `org.springframework.core.env.SystemEnvironmentPropertySource`。
#[derive(Debug, Clone)]
pub struct SystemEnvironmentPropertySource {
    name: String,
}

impl SystemEnvironmentPropertySource {
    /// 创建新的系统环境变量属性源。
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl PropertySource for SystemEnvironmentPropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// 属性源集合。
///
/// 对标 Spring `org.springframework.core.env.PropertySources`。
pub struct PropertySources {
    /// 属性源列表（按优先级排序，先添加的优先级更高）
    sources: Vec<Box<dyn PropertySource>>,
}

impl PropertySources {
    /// 创建空的属性源集合。
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// 添加属性源（添加到最高优先级位置）。
    pub fn add_first(&mut self, source: Box<dyn PropertySource>) {
        self.sources.insert(0, source);
    }

    /// 添加属性源（添加到最低优先级位置）。
    pub fn add_last(&mut self, source: Box<dyn PropertySource>) {
        self.sources.push(source);
    }

    /// 获取属性值（按优先级顺序查找）。
    pub fn get_property(&self, key: &str) -> Option<String> {
        for source in &self.sources {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
    }

    /// 检查属性是否存在。
    pub fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// 获取属性源数量。
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl Default for PropertySources {
    fn default() -> Self {
        Self::new()
    }
}

/// 环境 trait。
///
/// 对标 Spring `org.springframework.core.env.Environment`。
pub trait Environment: Send + Sync {
    /// 获取属性值。
    fn get_property(&self, key: &str) -> Option<String>;

    /// 获取属性值，如果不存在则返回默认值。
    fn get_property_with_default(&self, key: &str, default: &str) -> String {
        self.get_property(key).unwrap_or_else(|| default.to_string())
    }

    /// 检查属性是否存在。
    fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// 获取活跃 profile 列表。
    fn get_active_profiles(&self) -> Vec<String>;

    /// 获取默认 profile 列表。
    fn get_default_profiles(&self) -> Vec<String>;

    /// 检查是否接受指定的 profile。
    fn accepts_profiles(&self, profiles: &[&str]) -> bool {
        let active = self.get_active_profiles();
        profiles.iter().any(|p| active.contains(&p.to_string()))
    }
}

/// 标准环境实现。
///
/// 对标 Spring `org.springframework.core.env.StandardEnvironment`。
pub struct StandardEnvironment {
    /// 属性源集合
    property_sources: PropertySources,
    /// 活跃 profile
    active_profiles: Vec<String>,
    /// 默认 profile
    default_profiles: Vec<String>,
}

impl StandardEnvironment {
    /// 创建新的标准环境。
    pub fn new() -> Self {
        let mut property_sources = PropertySources::new();

        // 添加系统属性（对标 Spring 的 SystemPropertiesPropertySource）
        property_sources.add_last(Box::new(MapPropertySource::new(
            "systemProperties",
            Self::collect_system_properties(),
        )));

        // 添加系统环境变量（对标 Spring 的 SystemEnvironmentPropertySource）
        property_sources.add_last(Box::new(SystemEnvironmentPropertySource::new(
            "systemEnvironment",
        )));

        Self {
            property_sources,
            active_profiles: Vec::new(),
            default_profiles: vec!["default".to_string()],
        }
    }

    /// 添加自定义属性源。
    pub fn add_property_source(&mut self, source: Box<dyn PropertySource>) {
        self.property_sources.add_last(source);
    }

    /// 设置活跃 profile。
    pub fn set_active_profiles(&mut self, profiles: Vec<String>) {
        self.active_profiles = profiles;
    }

    /// 收集系统属性。
    fn collect_system_properties() -> HashMap<String, String> {
        let mut properties = HashMap::new();
        // 收集常见的系统属性
        if let Ok(home) = std::env::var("HOME") {
            properties.insert("user.home".to_string(), home);
        }
        if let Ok(user) = std::env::var("USER") {
            properties.insert("user.name".to_string(), user);
        }
        if let Ok(lang) = std::env::var("LANG") {
            properties.insert("user.language".to_string(), lang);
        }
        properties
    }
}

impl Default for StandardEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment for StandardEnvironment {
    fn get_property(&self, key: &str) -> Option<String> {
        self.property_sources.get_property(key)
    }

    fn get_active_profiles(&self) -> Vec<String> {
        self.active_profiles.clone()
    }

    fn get_default_profiles(&self) -> Vec<String> {
        self.default_profiles.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_property_source_basic() {
        let mut properties = HashMap::new();
        properties.insert("key1".to_string(), "value1".to_string());
        properties.insert("key2".to_string(), "value2".to_string());

        let source = MapPropertySource::new("test", properties);
        assert_eq!(source.name(), "test");
        assert_eq!(source.get_property("key1"), Some("value1".to_string()));
        assert_eq!(source.get_property("key2"), Some("value2".to_string()));
        assert_eq!(source.get_property("key3"), None);
    }

    #[test]
    fn system_environment_property_source() {
        let source = SystemEnvironmentPropertySource::new("env");
        // PATH 环境变量应该存在
        assert!(source.get_property("PATH").is_some());
    }

    #[test]
    fn property_sources_priority() {
        let mut sources = PropertySources::new();

        let mut low = HashMap::new();
        low.insert("key".to_string(), "low".to_string());
        sources.add_last(Box::new(MapPropertySource::new("low", low)));

        let mut high = HashMap::new();
        high.insert("key".to_string(), "high".to_string());
        sources.add_first(Box::new(MapPropertySource::new("high", high)));

        assert_eq!(sources.get_property("key"), Some("high".to_string()));
    }

    #[test]
    fn standard_environment_has_system_properties() {
        let env = StandardEnvironment::new();
        // 系统属性应该存在
        assert!(env.get_property("user.home").is_some() || env.get_property("user.name").is_some());
    }

    #[test]
    fn standard_environment_default_profiles() {
        let env = StandardEnvironment::new();
        assert_eq!(env.get_default_profiles(), vec!["default".to_string()]);
    }

    #[test]
    fn standard_environment_active_profiles() {
        let mut env = StandardEnvironment::new();
        env.set_active_profiles(vec!["dev".to_string(), "test".to_string()]);
        assert_eq!(env.get_active_profiles(), vec!["dev".to_string(), "test".to_string()]);
        assert!(env.accepts_profiles(&["dev"]));
        assert!(!env.accepts_profiles(&["prod"]));
    }

    #[test]
    fn property_sources_add_first() {
        let mut sources = PropertySources::new();
        let mut low = HashMap::new();
        low.insert("key".to_string(), "low".to_string());
        sources.add_last(Box::new(MapPropertySource::new("low", low)));

        let mut high = HashMap::new();
        high.insert("key".to_string(), "high".to_string());
        sources.add_first(Box::new(MapPropertySource::new("high", high)));

        assert_eq!(sources.get_property("key"), Some("high".to_string()));
    }

    #[test]
    fn property_sources_add_last() {
        let mut sources = PropertySources::new();
        let mut first = HashMap::new();
        first.insert("key".to_string(), "first".to_string());
        sources.add_last(Box::new(MapPropertySource::new("first", first)));

        let mut second = HashMap::new();
        second.insert("key".to_string(), "second".to_string());
        sources.add_last(Box::new(MapPropertySource::new("second", second)));

        // 先添加的优先级更高
        assert_eq!(sources.get_property("key"), Some("first".to_string()));
    }

    #[test]
    fn property_sources_len() {
        let mut sources = PropertySources::new();
        assert_eq!(sources.len(), 0);
        assert!(sources.is_empty());

        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        sources.add_last(Box::new(MapPropertySource::new("test", props)));
        assert_eq!(sources.len(), 1);
        assert!(!sources.is_empty());
    }

    #[test]
    fn property_sources_contains_property() {
        let mut sources = PropertySources::new();
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        sources.add_last(Box::new(MapPropertySource::new("test", props)));

        assert!(sources.contains_property("key"));
        assert!(!sources.contains_property("missing"));
    }

    #[test]
    fn property_sources_get_property_missing() {
        let sources = PropertySources::new();
        assert_eq!(sources.get_property("missing"), None);
    }

    #[test]
    fn map_property_source_set_property() {
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        let mut source = MapPropertySource::new("test", props);
        assert_eq!(source.get_property("key"), Some("value".to_string()));

        source.set_property("key", "new_value");
        assert_eq!(source.get_property("key"), Some("new_value".to_string()));
    }

    #[test]
    fn map_property_source_remove_property() {
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        let mut source = MapPropertySource::new("test", props);
        assert!(source.contains_property("key"));

        source.remove_property("key");
        assert!(!source.contains_property("key"));
    }

    #[test]
    fn system_environment_property_source_contains_path() {
        let source = SystemEnvironmentPropertySource::new("env");
        assert!(source.get_property("PATH").is_some());
    }

    #[test]
    fn standard_environment_get_property() {
        let env = StandardEnvironment::new();
        // 系统属性应该存在
        assert!(env.get_property("user.home").is_some() || env.get_property("user.name").is_some());
    }

    #[test]
    fn standard_environment_get_property_with_default() {
        let env = StandardEnvironment::new();
        let value = env.get_property_with_default("nonexistent_key", "default_value");
        assert_eq!(value, "default_value");
    }

    #[test]
    fn standard_environment_contains_property() {
        let env = StandardEnvironment::new();
        assert!(env.contains_property("PATH"));
        assert!(!env.contains_property("nonexistent_key_12345"));
    }
}
