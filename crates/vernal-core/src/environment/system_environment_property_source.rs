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
