//! 环境抽象 trait。
//!
//! 对标 Spring `org.springframework.core.env.Environment`。

use super::PropertyResolver;

/// 环境抽象 trait。
///
/// 对应 Java: org.springframework.core.env.Environment
///
/// Spring 层次：`Environment extends PropertyResolver`——属性查找与占位符
/// 解析继承自 [`PropertyResolver`]，此处补充 profile 语义。
pub trait Environment: PropertyResolver {
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
