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

#[cfg(test)]
mod tests {
    use super::*;

    /// 对标 Spring `StandardEnvironment()` 默认构造:
    /// 默认 profile 为 `["default"]`, active 为空
    #[test]
    fn default_profile_is_default_and_active_is_empty() {
        let env = StandardEnvironment::new();
        assert_eq!(env.get_default_profiles(), vec!["default".to_string()]);
        assert!(env.get_active_profiles().is_empty());
    }

    /// 对标 Spring `setActiveProfiles(String...)` 替换活跃 profile 列表
    #[test]
    fn set_active_profiles_replaces_full_list() {
        let mut env = StandardEnvironment::new();
        env.set_active_profiles(vec!["dev".to_string(), "debug".to_string()]);
        assert_eq!(
            env.get_active_profiles(),
            vec!["dev".to_string(), "debug".to_string()]
        );
    }

    /// 对标 Spring `Environment.getProperty(key)`: 通过 systemProperties 源
    /// 读取到的 `user.home` / `user.name` / `user.language`
    #[test]
    fn get_property_reads_system_properties_via_property_sources() {
        let env = StandardEnvironment::new();
        // 单独读取每个系统属性以保证每个分支都被覆盖
        let home = env.get_property("user.home");
        let name = env.get_property("user.name");
        let lang = env.get_property("user.language");
        let has_any_system_prop = home.is_some() || name.is_some() || lang.is_some();
        assert!(
            has_any_system_prop,
            "should populate at least one standard user.* property from environment"
        );
        // 显式求值所有三个属性, 保证每个 get_property 都被调用
        let _ = (home, name, lang);
    }

    /// 对标 Spring `addPropertySource`: 用户自定义属性源优先级
    #[test]
    fn add_property_source_appends_custom_source() {
        use super::super::map_property_source::MapPropertySource;
        let mut env = StandardEnvironment::new();
        let mut custom = HashMap::new();
        custom.insert("custom.key".to_string(), "custom-value".to_string());
        env.add_property_source(Box::new(MapPropertySource::new("custom", custom)));
        assert_eq!(
            env.get_property("custom.key"),
            Some("custom-value".to_string())
        );
    }

    /// `Default::default()` 与 `new()` 行为一致
    #[test]
    fn default_impl_matches_new() {
        let from_new = StandardEnvironment::new();
        let from_default = StandardEnvironment::default();
        assert_eq!(from_new.get_default_profiles(), from_default.get_default_profiles());
        assert_eq!(from_new.get_active_profiles(), from_default.get_active_profiles());
    }
}
