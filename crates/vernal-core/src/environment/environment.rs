//! 环境抽象 trait。
//!
//! 对标 Spring `org.springframework.core.env.Environment`。

/// 环境抽象 trait。
///
/// 对应 Java: org.springframework.core.env.Environment
pub trait Environment: Send + Sync {
    /// 获取属性值。
    ///
    /// 对应 Java: `Environment#getProperty`
    fn get_property(&self, key: &str) -> Option<String>;

    /// 获取属性值，如果不存在则返回默认值。
    ///
    /// 对应 Java: `Environment#getProperty(String, String)`
    fn get_property_with_default(&self, key: &str, default: &str) -> String {
        self.get_property(key).unwrap_or_else(|| default.to_string())
    }

    /// 检查属性是否存在。
    ///
    /// 对应 Java: `Environment#containsProperty`
    fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// 获取活跃 profile 列表。
    ///
    /// 对应 Java: `Environment#getActiveProfiles`
    fn get_active_profiles(&self) -> Vec<String>;

    /// 获取默认 profile 列表。
    ///
    /// 对应 Java: `Environment#getDefaultProfiles`
    fn get_default_profiles(&self) -> Vec<String>;

    /// 检查是否接受指定的 profile。
    ///
    /// 对应 Java: `Environment#acceptsProfiles`
    fn accepts_profiles(&self, profiles: &[&str]) -> bool {
        let active = self.get_active_profiles();
        profiles.iter().any(|p| active.contains(&p.to_string()))
    }
}
