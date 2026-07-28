//! 标准环境实现。
//!
//! 对标 Spring `org.springframework.core.env.StandardEnvironment`。

use std::collections::HashMap;

use super::environment::Environment;
use super::map_property_source::MapPropertySource;
use super::property_source::PropertySource;
use super::system_environment_property_source::SystemEnvironmentPropertySource;

/// 标准环境实现。
///
/// 对应 Java: org.springframework.core.env.StandardEnvironment
pub struct StandardEnvironment {
    /// 属性源集合
    property_sources: super::property_sources::PropertySources,
    /// 活跃 profile
    active_profiles: Vec<String>,
    /// 默认 profile
    default_profiles: Vec<String>,
}

impl StandardEnvironment {
    /// 创建新的标准环境。
    ///
    /// 对应 Java: `StandardEnvironment()`
    #[must_use]
    pub fn new() -> Self {
        let mut property_sources = super::property_sources::PropertySources::new();

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
