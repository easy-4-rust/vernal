//! Environment — Spring 风格的 Environment trait。
//!
//! 对应 Java 类：`org.springframework.core.env.Environment`。
//!
//! 表示应用运行环境，组合 profile 管理与属性解析能力。

use crate::property_resolver::PropertyResolver;

/// Spring 风格的 Environment trait。
///
/// 对应 Spring 的 `Environment`。
///
/// 继承 `PropertyResolver`，额外提供 active/default profile 的查询能力。
pub trait Environment: PropertyResolver {
    /// 获取当前激活的 profile 名称集合。
    ///
    /// 对应 Spring 的 `getActiveProfiles()`。
    fn get_active_profiles(&self) -> Vec<String>;

    /// 获取默认 profile 名称集合。
    ///
    /// 对应 Spring 的 `getDefaultProfiles()`。
    fn get_default_profiles(&self) -> Vec<String>;

    /// 是否接受了（激活）给定 profile。
    ///
    /// 对应 Spring 的 `acceptsProfiles(String... profiles)`（已废弃形式的替代）。
    fn accepts_profiles(&self, profiles: &[String]) -> bool {
        let active = self.get_active_profiles();
        profiles.iter().any(|p| active.iter().any(|a| a == p))
    }

    /// 是否接受了（激活）单个 profile。
    fn accepts_profile(&self, profile: &str) -> bool {
        self.get_active_profiles().iter().any(|p| p == profile)
    }
}

/// 简单的 `Environment` 实现。
///
/// 内部使用 `SimpleEnvironmentStorage` 持有属性、profile 与占位符解析。
pub struct SimpleEnvironment {
    properties: std::collections::HashMap<String, String>,
    active_profiles: Vec<String>,
    default_profiles: Vec<String>,
}

impl std::fmt::Debug for SimpleEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleEnvironment")
            .field("property_count", &self.properties.len())
            .field("active_profiles", &self.active_profiles)
            .field("default_profiles", &self.default_profiles)
            .finish()
    }
}

impl Default for SimpleEnvironment {
    fn default() -> Self {
        Self {
            properties: std::collections::HashMap::new(),
            active_profiles: Vec::new(),
            default_profiles: vec!["default".to_owned()],
        }
    }
}

impl SimpleEnvironment {
    /// 创建空的简单环境（默认 profile 为 `"default"`）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置一个属性。
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// 设置激活的 profile 集合。
    pub fn set_active_profiles<I, S>(&mut self, profiles: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.active_profiles = profiles.into_iter().map(Into::into).collect();
    }

    /// 设置默认 profile 集合。
    pub fn set_default_profiles<I, S>(&mut self, profiles: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_profiles = profiles.into_iter().map(Into::into).collect();
    }

    /// 底层属性映射。
    pub fn properties(&self) -> &std::collections::HashMap<String, String> {
        &self.properties
    }

    fn active_for_query(&self) -> Vec<String> {
        if self.active_profiles.is_empty() {
            self.default_profiles.clone()
        } else {
            self.active_profiles.clone()
        }
    }
}

impl PropertyResolver for SimpleEnvironment {
    fn contains_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }

    fn resolve_placeholders(&self, text: &str) -> String {
        resolve_placeholders_internal(text, |name| self.properties.get(name).cloned())
    }

    fn resolve_required_placeholders(
        &self,
        text: &str,
    ) -> Result<String, crate::property_resolver::UnresolvedPlaceholderError> {
        let resolved = self.resolve_placeholders(text);
        if resolved.contains("${") {
            Err(crate::property_resolver::UnresolvedPlaceholderError::new(
                text.to_owned(),
            ))
        } else {
            Ok(resolved)
        }
    }
}

impl Environment for SimpleEnvironment {
    fn get_active_profiles(&self) -> Vec<String> {
        self.active_for_query()
    }

    fn get_default_profiles(&self) -> Vec<String> {
        self.default_profiles.clone()
    }
}

/// 使用一个属性查找函数解析字符串中的 `${...}` 占位符（公共工具）。
pub(crate) fn resolve_placeholders_internal(
    value: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> String {
    let prefix = "${";
    let suffix = "}";
    let mut current = value.to_owned();
    for _ in 0..16 {
        if !current.contains(prefix) {
            break;
        }
        let mut out = String::with_capacity(current.len());
        let mut rest = current.as_str();
        let mut changed = false;
        while let Some(start) = rest.find(prefix) {
            out.push_str(&rest[..start]);
            let after = &rest[start + prefix.len()..];
            if let Some(end) = after.find(suffix) {
                let name_raw = &after[..end];
                let (name, default) = match name_raw.split_once(':') {
                    Some((n, d)) => (n.trim(), Some(d)),
                    None => (name_raw.trim(), None),
                };
                match lookup(name) {
                    Some(v) => {
                        out.push_str(&v);
                        changed = true;
                    }
                    None => match default {
                        Some(d) => {
                            out.push_str(d);
                            changed = true;
                        }
                        None => {
                            out.push_str(prefix);
                            out.push_str(name_raw);
                            out.push_str(suffix);
                        }
                    },
                }
                rest = &after[end + suffix.len()..];
            } else {
                out.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
        out.push_str(rest);
        if !changed {
            break;
        }
        current = out;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_environment_profiles() {
        let mut env = SimpleEnvironment::new();
        assert_eq!(env.get_active_profiles(), vec!["default"]);
        env.set_active_profiles(["dev", "cloud"]);
        assert!(env.accepts_profile("dev"));
        assert!(!env.accepts_profile("default"));
        assert_eq!(env.get_default_profiles(), vec!["default"]);
    }

    #[test]
    fn test_simple_environment_properties() {
        let mut env = SimpleEnvironment::new();
        env.set_property("name", "vernal");
        assert!(env.contains_property("name"));
        assert_eq!(env.get_property("name"), Some("vernal".to_owned()));
        assert_eq!(env.get_property_or("missing", "fb"), "fb");
        assert!(env.get_required_property("missing").is_err());
    }

    #[test]
    fn test_simple_environment_resolve_placeholders() {
        let mut env = SimpleEnvironment::new();
        env.set_property("name", "vernal");
        env.set_property("greeting", "Hello ${name}!");
        assert_eq!(env.resolve_placeholders("${greeting}"), "Hello vernal!");
        assert_eq!(env.resolve_placeholders("${unknown:def}"), "def");
        assert!(env.resolve_required_placeholders("${unknown}").is_err());
    }
}
