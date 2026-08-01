//! 系统环境变量属性源。
//!
//! 对标 Spring `org.springframework.core.env.SystemEnvironmentPropertySource`。

use super::property_source::PropertySource;

/// 系统环境变量属性源。
///
/// 对应 Java: org.springframework.core.env.SystemEnvironmentPropertySource
#[derive(Debug, Clone)]
pub struct SystemEnvironmentPropertySource {
    name: String,
}

impl SystemEnvironmentPropertySource {
    /// 创建新的系统环境变量属性源。
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_name() {
        let src = SystemEnvironmentPropertySource::new("systemEnvironment");
        assert_eq!(src.name(), "systemEnvironment");
    }

    #[test]
    fn name_returns_stored_value() {
        let src = SystemEnvironmentPropertySource::new("custom-name");
        assert_eq!(src.name(), "custom-name");
    }

    #[test]
    fn get_property_reads_real_env_var() {
        // HOME is virtually always set on Unix/macOS
        let src = SystemEnvironmentPropertySource::new("test");
        let home = src.get_property("HOME");
        assert!(home.is_some(), "HOME should be set");
    }

    #[test]
    fn get_property_returns_none_for_missing_var() {
        let src = SystemEnvironmentPropertySource::new("test");
        assert!(src.get_property("VERNAL_TEST_NONEXISTENT_VAR_12345").is_none());
    }

    #[test]
    fn contains_property_delegates_to_get_property() {
        let src = SystemEnvironmentPropertySource::new("test");
        // PATH is typically present on all systems
        assert!(src.contains_property("PATH"));
        assert!(!src.contains_property("VERNAL_TEST_NONEXISTENT_VAR_12345"));
    }
}
