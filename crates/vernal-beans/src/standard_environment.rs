//! StandardEnvironment — 标准环境实现。
//!
//! 对应 Java 类：`org.springframework.core.env.StandardEnvironment`。
//!
//! 在 `AbstractEnvironment` 基础上预注册系统属性与系统环境变量属性源。

use std::collections::HashMap;

use crate::abstract_environment::AbstractEnvironment;
use crate::configurable_environment::ConfigurableEnvironment;
use crate::environment::Environment;
use crate::mutable_property_sources::MutablePropertySources;
use crate::property_resolver::PropertyResolver;
use crate::property_source::PropertySource;

/// 系统环境变量属性源名称。
///
/// 对应 Spring 的 `StandardEnvironment.SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME`。
pub const SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME: &str = "systemEnvironment";

/// 系统属性属性源名称。
///
/// 对应 Spring 的 `StandardEnvironment.SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME`。
pub const SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME: &str = "systemProperties";

/// Spring 风格的标准环境实现。
///
/// 对应 Spring 的 `StandardEnvironment`。
///
/// 预注册两个属性源：
///
/// - `systemProperties` — 来自进程参数 / JVM 等价的系统属性
/// - `systemEnvironment` — 来自 OS 的环境变量
///
/// 在 Rust 中，这两者分别通过 `build_system_properties` 与
/// `build_system_environment` 构造，默认实现从 `std::env` 读取（受限）。
#[derive(Debug)]
pub struct StandardEnvironment {
    inner: AbstractEnvironment,
}

impl Default for StandardEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardEnvironment {
    /// 创建标准环境，并预注册系统属性源与系统环境变量源。
    pub fn new() -> Self {
        Self::with_sources(build_system_properties(), build_system_environment())
    }

    /// 使用指定的系统属性与系统环境变量构造标准环境。
    pub fn with_sources(
        system_properties: HashMap<String, String>,
        system_environment: HashMap<String, String>,
    ) -> Self {
        let mut inner = AbstractEnvironment::new();
        let mut sources = MutablePropertySources::new();
        // systemProperties 优先级高于 systemEnvironment（与 Spring 一致）。
        sources.add_last(PropertySource::with_source(
            SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME,
            system_properties.clone(),
        ));
        sources.add_last(PropertySource::with_source(
            SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME,
            system_environment.clone(),
        ));
        inner.set_property_sources(sources);
        inner.set_system_properties(system_properties);
        inner.set_system_environment(system_environment);
        Self { inner }
    }
}

impl PropertyResolver for StandardEnvironment {
    fn contains_property(&self, key: &str) -> bool {
        self.inner.contains_property(key)
    }
    fn get_property(&self, key: &str) -> Option<String> {
        self.inner.get_property(key)
    }
    fn resolve_placeholders(&self, text: &str) -> String {
        self.inner.resolve_placeholders(text)
    }
    fn resolve_required_placeholders(
        &self,
        text: &str,
    ) -> Result<String, crate::property_resolver::UnresolvedPlaceholderError> {
        self.inner.resolve_required_placeholders(text)
    }
}

impl Environment for StandardEnvironment {
    fn get_active_profiles(&self) -> Vec<String> {
        self.inner.get_active_profiles()
    }
    fn get_default_profiles(&self) -> Vec<String> {
        self.inner.get_default_profiles()
    }
}

impl ConfigurableEnvironment for StandardEnvironment {
    fn property_sources(&self) -> std::sync::Arc<std::sync::RwLock<MutablePropertySources>> {
        self.inner.property_sources()
    }
    fn set_active_profiles(&mut self, profiles: &[String]) {
        self.inner.set_active_profiles(profiles);
    }
    fn add_active_profile(&mut self, profile: &str) {
        self.inner.add_active_profile(profile);
    }
    fn set_default_profiles(&mut self, profiles: &[String]) {
        self.inner.set_default_profiles(profiles);
    }
    fn get_system_properties(&self) -> HashMap<String, String> {
        self.inner.get_system_properties()
    }
    fn get_system_environment(&self) -> HashMap<String, String> {
        self.inner.get_system_environment()
    }
}

/// 构造系统属性映射。
///
/// 默认从 `std::env::vars` 过滤得到非敏感的环境变量作为占位。
/// 在受限环境中返回空映射。
pub fn build_system_properties() -> HashMap<String, String> {
    // Spring 中 systemProperties 来自 JVM -D 参数。Rust 中无等价概念，
    // 此处返回空映射，留给上层填充。
    HashMap::new()
}

/// 构造系统环境变量映射。
///
/// 默认从 `std::env::vars` 读取 OS 环境变量。
pub fn build_system_environment() -> HashMap<String, String> {
    std::env::vars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_environment_default_sources_registered() {
        let env = StandardEnvironment::new();
        let sources = env.property_sources();
        let guard = sources.read().unwrap();
        assert!(guard.contains(SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME));
        assert!(guard.contains(SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME));
    }

    #[test]
    fn test_standard_environment_resolves_env_var() {
        // 设置一个临时环境变量（测试运行期间）。
        // 注意：env 变量修改可能在并发测试中不安全，此处仅当变量已存在时验证。
        let env = StandardEnvironment::new();
        if let Ok(value) = std::env::var("PATH") {
            let resolved = env.get_property("PATH");
            assert_eq!(resolved.as_deref(), Some(value.as_str()));
        }
    }
}
