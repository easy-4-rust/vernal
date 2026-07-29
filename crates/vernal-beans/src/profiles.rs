//! Profiles — Spring 风格的 profile 集合。
//!
//! 对应 Java 类：`org.springframework.core.env.Profiles` 周边概念。
//!
//! 管理 active 与 default profile 集合，提供查询与变更能力。

use crate::profile::Profile;

/// 默认 profile 名称。
///
/// 对应 Spring 的 `AbstractEnvironment.RESERVED_DEFAULT_PROFILE_NAME`。
pub const RESERVED_DEFAULT_PROFILE_NAME: &str = "default";

/// Spring 风格的 profile 集合。
///
/// 对应 Spring 的 profile 管理能力（`AbstractEnvironment` 中的 active/default profile）。
///
/// 维护两个集合：显式激活的 profile 与默认 profile。当未显式激活任何
/// profile 时，默认 profile 生效。
#[derive(Debug, Clone, Default)]
pub struct Profiles {
    active: Vec<Profile>,
    default_profiles: Vec<Profile>,
}

impl Profiles {
    /// 创建空的 profile 集合，默认 profile 为 `"default"`。
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            default_profiles: vec![Profile::new_default(RESERVED_DEFAULT_PROFILE_NAME)],
        }
    }

    /// 获取当前生效的 profile 名集合。
    ///
    /// 如果显式激活了 profile，则返回激活集合；
    /// 否则返回默认 profile 集合。
    pub fn get_active(&self) -> Vec<&str> {
        if self.active.is_empty() {
            self.default_profiles.iter().map(|p| p.name()).collect()
        } else {
            self.active.iter().map(|p| p.name()).collect()
        }
    }

    /// 获取默认 profile 名集合。
    pub fn get_default(&self) -> Vec<&str> {
        self.default_profiles.iter().map(|p| p.name()).collect()
    }

    /// 判断给定 profile 名称是否处于激活态（含默认回退）。
    pub fn is_active(&self, profile_name: &str) -> bool {
        self.get_active().iter().any(|p| *p == profile_name)
    }

    /// 添加一个显式激活的 profile。
    pub fn add(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.active.iter().any(|p| p.name() == name) {
            self.active.push(Profile::new(name));
        }
    }

    /// 添加一个默认 profile。
    pub fn add_default(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.default_profiles.iter().any(|p| p.name() == name) {
            self.default_profiles.push(Profile::new_default(name));
        }
    }

    /// 移除一个显式激活的 profile。
    pub fn remove(&mut self, name: &str) {
        self.active.retain(|p| p.name() != name);
    }

    /// 设置显式激活的 profile 集合（替换）。
    pub fn set_active(&mut self, names: &[String]) {
        self.active = names.iter().map(|n| Profile::new(n.clone())).collect();
    }

    /// 设置默认 profile 集合（替换）。
    pub fn set_default(&mut self, names: &[String]) {
        self.default_profiles = names
            .iter()
            .map(|n| Profile::new_default(n.clone()))
            .collect();
    }

    /// 显式激活的 profile 数量。
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// 是否显式激活了至少一个 profile。
    pub fn has_explicit_active(&self) -> bool {
        !self.active.is_empty()
    }

    /// 接受指定 profile 名集合作为激活集（仅添加已配置的默认中存在的项）。
    ///
    /// 这里简化为直接接受任意名称集合。
    pub fn accept(&mut self, names: &[String]) {
        self.set_active(names);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_active() {
        let profiles = Profiles::new();
        assert!(!profiles.has_explicit_active());
        assert!(profiles.is_active("default"));
        assert_eq!(profiles.get_active(), vec!["default"]);
    }

    #[test]
    fn test_explicit_active() {
        let mut profiles = Profiles::new();
        profiles.add("dev");
        profiles.add("dev"); // 去重
        profiles.add("cloud");
        assert_eq!(profiles.active_count(), 2);
        assert!(profiles.is_active("dev"));
        assert!(!profiles.is_active("default"));

        profiles.remove("dev");
        assert!(!profiles.is_active("dev"));
        assert_eq!(profiles.active_count(), 1);
    }

    #[test]
    fn test_set_active_replaces() {
        let mut profiles = Profiles::new();
        profiles.add("dev");
        profiles.set_active(&["prod".to_owned()]);
        assert!(!profiles.is_active("dev"));
        assert!(profiles.is_active("prod"));
    }
}
