//! ConfigurableEnvironment — 可配置环境 trait。
//!
//! 对应 Java 类：`org.springframework.core.env.ConfigurableEnvironment`。
//!
//! 扩展 `Environment`，提供对 `MutablePropertySources`、系统属性与系统环境变量的访问。

use std::collections::HashMap;
use std::sync::Arc;

use crate::environment::Environment;
use crate::mutable_property_sources::MutablePropertySources;

/// Spring 风格的可配置环境 trait。
///
/// 对应 Spring 的 `ConfigurableEnvironment`。
///
/// 在 `Environment` 基础上，允许配置应用设置 `MutablePropertySources`，
/// 并暴露系统属性（`-D` 参数）与系统环境变量（OS 环境变量）。
pub trait ConfigurableEnvironment: Environment {
    /// 获取可变属性源集合。
    ///
    /// 对应 Spring 的 `getPropertySources()`。
    fn property_sources(&self) -> Arc<std::sync::RwLock<MutablePropertySources>>;

    /// 设置激活的 profile 集合。
    ///
    /// 对应 Spring 的 `setActiveProfiles(String... profiles)`。
    fn set_active_profiles(&mut self, profiles: &[String]);

    /// 添加一个 profile 到激活集合。
    ///
    /// 对应 Spring 的 `addActiveProfile(String profile)`。
    fn add_active_profile(&mut self, profile: &str);

    /// 设置默认 profile 集合。
    ///
    /// 对应 Spring 的 `setDefaultProfiles(String... profiles)`。
    fn set_default_profiles(&mut self, profiles: &[String]);

    /// 合并另一个环境的属性源（用于父子环境）。
    ///
    /// 对应 Spring 的 `merge(ConfigurableEnvironment parent)`。
    fn merge(&mut self, parent: &dyn ConfigurableEnvironment) {
        let parent_sources = parent.property_sources();
        let guard = parent_sources.read().expect("property sources poisoned");
        let mine = self.property_sources();
        let mut mine_guard = mine.write().expect("property sources poisoned");
        for name in guard.names() {
            if !mine_guard.contains(name) {
                if let Some(ps) = guard.get(name) {
                    mine_guard.add_last(ps.clone());
                }
            }
        }
    }

    /// 获取系统属性（对应 Spring 的 `getSystemProperties()`）。
    ///
    /// 默认返回空映射；具体实现可填充真实的 JVM/进程参数。
    fn get_system_properties(&self) -> HashMap<String, String>;

    /// 获取系统环境变量（对应 Spring 的 `getSystemEnvironment()`）。
    ///
    /// 默认返回空映射；具体实现可填充真实的 OS 环境变量。
    fn get_system_environment(&self) -> HashMap<String, String>;
}
