//! Profile — Spring 风格的命名 profile。
//!
//! 对应 Java 类：`org.springframework.core.env.Profile` 周边概念。
//!
//! 表示一个具名的 profile，可以标记为默认 profile。

/// Spring 风格的命名 profile。
///
/// 对应 Spring 的 profile 概念。
///
/// Profile 用于按环境（如 `dev`、`test`、`prod`）条件化地激活配置。
#[derive(Debug, Clone)]
pub struct Profile {
    /// profile 名称。
    name: String,
    /// 是否为默认 profile。
    default_profile: bool,
}

impl Profile {
    /// 创建普通 profile。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default_profile: false,
        }
    }

    /// 创建默认 profile。
    pub fn new_default(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default_profile: true,
        }
    }

    /// 获取 profile 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 是否为默认 profile。
    pub fn is_default(&self) -> bool {
        self.default_profile
    }

    /// 设置是否为默认 profile。
    pub fn set_default(&mut self, default_profile: bool) {
        self.default_profile = default_profile;
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Profile {}

impl std::hash::Hash for Profile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialOrd for Profile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Profile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile() {
        let p = Profile::new("dev");
        assert_eq!(p.name(), "dev");
        assert!(!p.is_default());

        let d = Profile::new_default("default");
        assert!(d.is_default());
    }

    #[test]
    fn test_profile_equality_by_name() {
        let a = Profile::new("prod");
        let b = Profile::new_default("prod");
        assert_eq!(a, b);
    }
}
