//! 可配置环境契约。
//!
//! 对标 Spring `org.springframework.core.env.ConfigurableEnvironment`。

use super::{Environment, MutablePropertySources};

/// 可配置环境契约。
///
/// 对应 Java: org.springframework.core.env.ConfigurableEnvironment
///
/// Spring 语义：在 `Environment` 之上增加 profile 修改、属性源集合访问与
/// 环境合并能力。
pub trait ConfigurableEnvironment: Environment {
    /// 替换活跃 profile 列表。
    ///
    /// 对应 Java: `ConfigurableEnvironment#setActiveProfiles(String...)`
    fn set_active_profiles(&mut self, profiles: Vec<String>);

    /// 替换默认 profile 列表。
    ///
    /// 对应 Java: `ConfigurableEnvironment#setDefaultProfiles(String...)`
    fn set_default_profiles(&mut self, profiles: Vec<String>);

    /// 返回可变属性源集合。
    ///
    /// 对应 Java: `ConfigurableEnvironment#getPropertySources()`
    fn property_sources(&mut self) -> &mut MutablePropertySources;

    /// 合并另一个环境的 profile（属性源合并受 Rust 借用限制，
    /// 由调用方通过 [`Self::property_sources`] 显式搬运）。
    ///
    /// 对应 Java: `ConfigurableEnvironment#merge(ConfigurableEnvironment)`
    fn merge(&mut self, other: &mut dyn ConfigurableEnvironment);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::AbstractEnvironment;

    #[test]
    fn abstract_environment_satisfies_contract() {
        // D 类（重构安全）：`AbstractEnvironment` 实现该契约
        fn assert_configurable<T: ConfigurableEnvironment>() {}
        assert_configurable::<AbstractEnvironment>();
    }
}
