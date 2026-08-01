//! Profile 契约。
//!
//! 对标 Spring `org.springframework.core.env.Profiles`。

/// Profile 匹配契约。
///
/// 对应 Java: org.springframework.core.env.Profiles
///
/// Spring 语义：对活跃 profile 列表做布尔匹配（由 [`crate::env::ProfilesParser`]
/// 解析表达式生成）。
pub trait Profiles: Send + Sync {
    /// 判断是否匹配给定的活跃 profile 列表。
    ///
    /// 对应 Java: `Profiles#matches(String...)`
    fn matches(&self, active_profiles: &[&str]) -> bool;
}

/// 单个 profile 名的匹配实现（表达式叶子节点）。
pub struct SingleProfile {
    name: String,
}

impl SingleProfile {
    /// 创建单 profile 匹配器。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Profiles for SingleProfile {
    fn matches(&self, active_profiles: &[&str]) -> bool {
        active_profiles.iter().any(|p| *p == self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_profile_matches_when_active() {
        // A 类（合同对齐）：对标 Spring 单 profile 匹配
        let profile = SingleProfile::new("dev");
        assert!(profile.matches(&["dev", "test"]));
        assert!(!profile.matches(&["prod"]));
        assert!(!profile.matches(&[]));
    }
}
