//! 抽象环境。
//!
//! 对标 Spring `org.springframework.core.env.AbstractEnvironment`。

use super::PropertyResolver;
use super::configurable_environment::ConfigurableEnvironment;
use super::environment::Environment;
use super::mutable_property_sources::MutablePropertySources;
use super::property_sources_property_resolver::PropertySourcesPropertyResolver;

/// 抽象环境。
///
/// 对应 Java: org.springframework.core.env.AbstractEnvironment
///
/// Spring 语义：`ConfigurableEnvironment` 的基础实现——持有
/// `MutablePropertySources` 与 active/default profiles，`getProperty` 经由
/// `PropertySourcesPropertyResolver` 按优先级解析（含占位符）。
#[derive(Default)]
pub struct AbstractEnvironment {
    property_sources: MutablePropertySources,
    active_profiles: Vec<String>,
    default_profiles: Vec<String>,
    /// 活跃 profile 是否被显式设置（对标 Spring explicitActiveProfiles）。
    active_profiles_explicit: bool,
    /// 默认 profile 是否被显式设置（对标 Spring explicitDefaultProfiles）。
    default_profiles_explicit: bool,
}

impl AbstractEnvironment {
    /// 创建空环境（无属性源，默认 profile 为 `["default"]`）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            property_sources: MutablePropertySources::new(),
            active_profiles: Vec::new(),
            default_profiles: vec!["default".to_string()],
            active_profiles_explicit: false,
            default_profiles_explicit: false,
        }
    }

    /// 构造按当前来源解析属性的解析器。
    fn resolver(&self) -> PropertySourcesPropertyResolver<'_> {
        PropertySourcesPropertyResolver::new(&self.property_sources)
    }
}

impl PropertyResolver for AbstractEnvironment {
    fn get_property(&self, key: &str) -> Option<String> {
        self.resolver().get_property(key)
    }

    fn resolve_placeholders_inner(&self, text: &str, ignore_unresolvable: bool) -> String {
        self.resolver()
            .resolve_placeholders_inner(text, ignore_unresolvable)
    }
}

impl Environment for AbstractEnvironment {
    fn get_active_profiles(&self) -> Vec<String> {
        self.active_profiles.clone()
    }

    fn get_default_profiles(&self) -> Vec<String> {
        self.default_profiles.clone()
    }
}

impl ConfigurableEnvironment for AbstractEnvironment {
    fn set_active_profiles(&mut self, profiles: Vec<String>) {
        self.active_profiles = profiles;
        self.active_profiles_explicit = true;
    }

    fn set_default_profiles(&mut self, profiles: Vec<String>) {
        self.default_profiles = profiles;
        self.default_profiles_explicit = true;
    }

    fn property_sources(&mut self) -> &mut MutablePropertySources {
        &mut self.property_sources
    }

    fn merge(&mut self, other: &mut dyn ConfigurableEnvironment) {
        // 对标 Spring merge: 仅当本环境未显式设置 profile 时继承对方；
        // 属性源由调用方经 property_sources() 显式搬运（Rust 借用限制）
        let other_active = other.get_active_profiles();
        if !self.active_profiles_explicit && !other_active.is_empty() {
            self.active_profiles = other_active;
        }
        let other_default = other.get_default_profiles();
        if !self.default_profiles_explicit && !other_default.is_empty() {
            self.default_profiles = other_default;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MapPropertySource;

    #[test]
    fn default_profiles_and_empty_active() {
        // A 类（合同对齐）：对标 Spring 默认 profile 语义
        let env = AbstractEnvironment::new();
        assert_eq!(env.get_default_profiles(), vec!["default".to_string()]);
        assert!(env.get_active_profiles().is_empty());
    }

    #[test]
    fn resolves_property_through_sources() {
        // A 类（合同对齐）：对标 Spring getProperty 经属性源解析
        let mut env = AbstractEnvironment::new();
        let mut map = std::collections::HashMap::new();
        map.insert("app.name".to_string(), "vernal".to_string());
        env.property_sources()
            .add_last(Box::new(MapPropertySource::new("app", map)));
        assert_eq!(env.get_property("app.name").as_deref(), Some("vernal"));
        assert!(env.contains_property("app.name"));
        assert_eq!(env.get_property_with_default("nope", "d"), "d");
    }

    #[test]
    fn resolves_placeholders_from_sources() {
        // A 类（合同对齐）：对标 Spring resolvePlaceholders
        let mut env = AbstractEnvironment::new();
        let mut map = std::collections::HashMap::new();
        map.insert("version".to_string(), "1.0".to_string());
        env.property_sources()
            .add_last(Box::new(MapPropertySource::new("app", map)));
        assert_eq!(env.resolve_placeholders("v${version}"), "v1.0");
    }

    #[test]
    fn set_and_merge_profiles() {
        // B 类（边界行为）：对标 Spring setActiveProfiles / merge
        let mut env = AbstractEnvironment::new();
        env.set_active_profiles(vec!["dev".to_string()]);
        assert_eq!(env.get_active_profiles(), vec!["dev".to_string()]);

        let mut other = AbstractEnvironment::new();
        other.set_active_profiles(vec!["prod".to_string()]);
        other.set_default_profiles(vec!["custom".to_string()]);
        let mut merged = AbstractEnvironment::new();
        merged.merge(&mut other);
        assert_eq!(merged.get_active_profiles(), vec!["prod".to_string()]);
        assert_eq!(merged.get_default_profiles(), vec!["custom".to_string()]);
    }

    #[test]
    fn required_property_via_resolver() {
        // C 类（错误路径）：对标 Spring getRequiredProperty
        let env = AbstractEnvironment::new();
        assert!(env.get_required_property("missing").is_err());
    }
}
