//! AbstractEnvironment — 环境抽象基类。
//!
//! 对应 Java 类：`org.springframework.core.env.AbstractEnvironment`。
//!
//! 提供 profile 管理与属性源管理的通用实现，供具体子类复用。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::configurable_environment::ConfigurableEnvironment;
use crate::environment::resolve_placeholders_internal;
use crate::mutable_property_sources::MutablePropertySources;
use crate::profiles::{Profiles, RESERVED_DEFAULT_PROFILE_NAME};
use crate::property_resolver::{
    MissingRequiredPropertyError, PropertyResolver, UnresolvedPlaceholderError,
};

/// Spring 风格的环境抽象基类。
///
/// 对应 Spring 的 `AbstractEnvironment`。
///
/// 持有 `MutablePropertySources` 与 `Profiles`，提供完整的 profile 管理
/// 与属性解析能力。子类通过覆盖 `system_properties` / `system_environment`
/// 注入具体的系统属性与环境变量。
pub struct AbstractEnvironment {
    property_sources: Arc<RwLock<MutablePropertySources>>,
    profiles: Profiles,
    system_properties: HashMap<String, String>,
    system_environment: HashMap<String, String>,
}

impl std::fmt::Debug for AbstractEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.property_sources.read().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("AbstractEnvironment")
            .field("property_source_count", &count)
            .field("active_profiles", &self.profiles.get_active())
            .field("default_profiles", &self.profiles.get_default())
            .finish()
    }
}

impl Default for AbstractEnvironment {
    fn default() -> Self {
        Self {
            property_sources: Arc::new(RwLock::new(MutablePropertySources::new())),
            profiles: Profiles::new(),
            system_properties: HashMap::new(),
            system_environment: HashMap::new(),
        }
    }
}

impl AbstractEnvironment {
    /// 创建空的抽象环境。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置系统属性。
    pub fn set_system_properties(&mut self, props: HashMap<String, String>) {
        self.system_properties = props;
    }

    /// 设置系统环境变量。
    pub fn set_system_environment(&mut self, env: HashMap<String, String>) {
        self.system_environment = env;
    }

    /// 获取 profile 管理器的引用。
    pub fn profiles(&self) -> &Profiles {
        &self.profiles
    }

    /// 获取 profile 管理器的可变引用。
    pub fn profiles_mut(&mut self) -> &mut Profiles {
        &mut self.profiles
    }

    /// 自定义属性源（替换）。
    pub fn set_property_sources(&mut self, sources: MutablePropertySources) {
        self.property_sources = Arc::new(RwLock::new(sources));
    }

    /// 计算当前生效的 profile 名集合。
    fn active_for_query(&self) -> Vec<String> {
        if self.profiles.has_explicit_active() {
            self.profiles
                .get_active()
                .into_iter()
                .map(String::from)
                .collect()
        } else {
            self.profiles
                .get_default()
                .into_iter()
                .map(String::from)
                .collect()
        }
    }
}

impl PropertyResolver for AbstractEnvironment {
    fn contains_property(&self, key: &str) -> bool {
        if let Ok(guard) = self.property_sources.read() {
            guard.get_property(key).is_some()
                || self.system_properties.contains_key(key)
                || self.system_environment.contains_key(key)
        } else {
            false
        }
    }

    fn get_property(&self, key: &str) -> Option<String> {
        if let Ok(guard) = self.property_sources.read() {
            if let Some(v) = guard.get_property(key) {
                return Some(v.to_owned());
            }
        }
        if let Some(v) = self.system_properties.get(key) {
            return Some(v.clone());
        }
        self.system_environment.get(key).cloned()
    }

    fn get_property_or(&self, key: &str, default_value: &str) -> String {
        self.get_property(key)
            .unwrap_or_else(|| default_value.to_owned())
    }

    fn get_required_property(&self, key: &str) -> Result<String, MissingRequiredPropertyError> {
        self.get_property(key)
            .ok_or_else(|| MissingRequiredPropertyError::new(key.to_owned()))
    }

    fn resolve_placeholders(&self, text: &str) -> String {
        let property_sources = self.property_sources.clone();
        resolve_placeholders_internal(text, |name| {
            let guard = property_sources.read().ok()?;
            guard.get_property(name).map(str::to_owned)
        })
    }

    fn resolve_required_placeholders(
        &self,
        text: &str,
    ) -> Result<String, UnresolvedPlaceholderError> {
        let resolved = self.resolve_placeholders(text);
        if resolved.contains("${") {
            Err(UnresolvedPlaceholderError::new(text.to_owned()))
        } else {
            Ok(resolved)
        }
    }
}

impl crate::environment::Environment for AbstractEnvironment {
    fn get_active_profiles(&self) -> Vec<String> {
        self.active_for_query()
    }

    fn get_default_profiles(&self) -> Vec<String> {
        self.profiles
            .get_default()
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn accepts_profiles(&self, profiles: &[String]) -> bool {
        let active = self.active_for_query();
        profiles.iter().any(|p| active.iter().any(|a| a == p))
    }
}

impl ConfigurableEnvironment for AbstractEnvironment {
    fn property_sources(&self) -> Arc<RwLock<MutablePropertySources>> {
        self.property_sources.clone()
    }

    fn set_active_profiles(&mut self, profiles: &[String]) {
        self.profiles.set_active(profiles);
    }

    fn add_active_profile(&mut self, profile: &str) {
        self.profiles.add(profile);
    }

    fn set_default_profiles(&mut self, profiles: &[String]) {
        self.profiles.set_default(profiles);
    }

    fn get_system_properties(&self) -> HashMap<String, String> {
        self.system_properties.clone()
    }

    fn get_system_environment(&self) -> HashMap<String, String> {
        self.system_environment.clone()
    }
}

/// 用于子类设置默认 profile 的便捷常量。
pub fn default_profile_name() -> &'static str {
    RESERVED_DEFAULT_PROFILE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::property_resolver::PropertyResolver;
    use crate::property_source::PropertySource;

    #[test]
    fn test_abstract_environment_profiles_and_properties() {
        let mut env = AbstractEnvironment::new();
        {
            let mut sources = env.property_sources.write().unwrap();
            sources.add_first(PropertySource::from_pairs(
                "app",
                [("name", "vernal"), ("greeting", "Hi ${name}")],
            ));
        }
        env.set_active_profiles(&["dev".to_owned()]);
        assert_eq!(env.get_active_profiles(), vec!["dev"]);
        assert!(env.accepts_profile("dev"));
        assert_eq!(env.get_property("name"), Some("vernal".to_owned()));
        assert_eq!(env.resolve_placeholders("${greeting}"), "Hi vernal");
    }

    #[test]
    fn test_default_profile_fallback() {
        let env = AbstractEnvironment::new();
        assert_eq!(env.get_active_profiles(), vec!["default"]);
    }

    #[test]
    fn test_merge() {
        let parent = AbstractEnvironment::new();
        {
            let mut sources = parent.property_sources.write().unwrap();
            sources.add_last(PropertySource::from_pairs("parent", [("k", "1")]));
        }
        let mut child = AbstractEnvironment::new();
        child.merge(&parent);
        assert_eq!(child.get_property("k"), Some("1".to_owned()));
    }
}
