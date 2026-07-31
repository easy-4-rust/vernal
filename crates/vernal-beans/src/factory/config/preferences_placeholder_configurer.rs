//! PreferencesPlaceholderConfigurer — 对应 Spring `org.springframework.beans.factory.config.PreferencesPlaceholderConfigurer`。
//!
//! 首选项占位符配置器。

use std::collections::HashMap;

/// 首选项占位符配置器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PreferencesPlaceholderConfigurer`。
///
/// 使用 Java Preferences API 作为占位符值的来源。
/// 在 Rust 实现中，使用 HashMap 模拟首选项存储。
///
/// ## 使用场景
///
/// - 用户/系统首选项配置
/// - 跨应用共享配置
/// - 操作系统级别的配置
#[derive(Debug)]
pub struct PreferencesPlaceholderConfigurer {
    /// 首选项存储。
    preferences: HashMap<String, String>,
    /// 系统首选项。
    system_preferences: HashMap<String, String>,
    /// 用户首选项。
    user_preferences: HashMap<String, String>,
    /// 占位符前缀。
    placeholder_prefix: String,
    /// 占位符后缀。
    placeholder_suffix: String,
}

impl PreferencesPlaceholderConfigurer {
    /// 创建新的 PreferencesPlaceholderConfigurer。
    pub fn new() -> Self {
        Self {
            preferences: HashMap::new(),
            system_preferences: HashMap::new(),
            user_preferences: HashMap::new(),
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
        }
    }

    /// 设置首选项。
    pub fn set_preference(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.preferences.insert(key.into(), value.into());
    }

    /// 获取首选项值。
    pub fn get_preference(&self, key: &str) -> Option<&str> {
        self.preferences.get(key).map(|s| s.as_str())
    }

    /// 设置系统首选项。
    pub fn set_system_preference(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.system_preferences.insert(key.into(), value.into());
    }

    /// 获取系统首选项值。
    pub fn get_system_preference(&self, key: &str) -> Option<&str> {
        self.system_preferences.get(key).map(|s| s.as_str())
    }

    /// 设置用户首选项。
    pub fn set_user_preference(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.user_preferences.insert(key.into(), value.into());
    }

    /// 获取用户首选项值。
    pub fn get_user_preference(&self, key: &str) -> Option<&str> {
        self.user_preferences.get(key).map(|s| s.as_str())
    }

    /// 设置占位符前缀。
    pub fn set_placeholder_prefix(&mut self, prefix: impl Into<String>) {
        self.placeholder_prefix = prefix.into();
    }

    /// 设置占位符后缀。
    pub fn set_placeholder_suffix(&mut self, suffix: impl Into<String>) {
        self.placeholder_suffix = suffix.into();
    }

    /// 解析占位符（按优先级：用户首选项 > 系统首选项 > 普通首选项）。
    pub fn resolve_placeholder(&self, key: &str) -> Option<&str> {
        self.get_user_preference(key)
            .or_else(|| self.get_system_preference(key))
            .or_else(|| self.get_preference(key))
    }

    /// 解析文本中的占位符。
    pub fn resolve_placeholders(&self, text: &str) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find(&self.placeholder_prefix) {
            if let Some(end) = result[start..].find(&self.placeholder_suffix) {
                let key = &result[start + self.placeholder_prefix.len()..start + end];
                if let Some(resolved) = self.resolve_placeholder(key) {
                    let resolved = resolved.to_string();
                    result = format!(
                        "{}{}{}",
                        &result[..start],
                        resolved,
                        &result[start + end + self.placeholder_suffix.len()..]
                    );
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }
}

impl Default for PreferencesPlaceholderConfigurer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preferences_placeholder_configurer_new() {
        let configurer = PreferencesPlaceholderConfigurer::new();
        assert!(configurer.preferences.is_empty());
        assert!(configurer.system_preferences.is_empty());
        assert!(configurer.user_preferences.is_empty());
    }

    #[test]
    fn test_preferences_placeholder_configurer_resolve_priority() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("key1", "default_value");
        configurer.set_system_preference("key1", "system_value");
        configurer.set_user_preference("key1", "user_value");

        // 用户首选项优先级最高
        assert_eq!(configurer.resolve_placeholder("key1"), Some("user_value"));
    }

    #[test]
    fn test_preferences_placeholder_configurer_resolve_text() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("app.name", "MyApp");
        configurer.set_system_preference("app.version", "1.0.0");

        let result = configurer.resolve_placeholders("${app.name} v${app.version}");
        assert_eq!(result, "MyApp v1.0.0");
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn default_trait_creates_correctly() {
        let configurer = PreferencesPlaceholderConfigurer::default();
        assert!(configurer.preferences.is_empty());
        assert!(configurer.system_preferences.is_empty());
        assert!(configurer.user_preferences.is_empty());
    }

    #[test]
    fn set_and_get_preference() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("key", "value");
        assert_eq!(configurer.get_preference("key"), Some("value"));
        assert_eq!(configurer.get_preference("missing"), None);
    }

    #[test]
    fn set_and_get_system_preference() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_system_preference("sys_key", "sys_value");
        assert_eq!(configurer.get_system_preference("sys_key"), Some("sys_value"));
        assert_eq!(configurer.get_system_preference("missing"), None);
    }

    #[test]
    fn set_and_get_user_preference() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_user_preference("user_key", "user_value");
        assert_eq!(configurer.get_user_preference("user_key"), Some("user_value"));
        assert_eq!(configurer.get_user_preference("missing"), None);
    }

    #[test]
    fn resolve_priority_system_over_default() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("key", "default");
        configurer.set_system_preference("key", "system");
        // No user preference, so system should win
        assert_eq!(configurer.resolve_placeholder("key"), Some("system"));
    }

    #[test]
    fn resolve_priority_default_fallback() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("key", "default");
        // No system or user preference
        assert_eq!(configurer.resolve_placeholder("key"), Some("default"));
    }

    #[test]
    fn resolve_placeholder_not_found() {
        let configurer = PreferencesPlaceholderConfigurer::new();
        assert!(configurer.resolve_placeholder("missing").is_none());
    }

    #[test]
    fn resolve_placeholders_no_placeholders() {
        let configurer = PreferencesPlaceholderConfigurer::new();
        let result = configurer.resolve_placeholders("no placeholders here");
        assert_eq!(result, "no placeholders here");
    }

    #[test]
    fn resolve_placeholders_unresolved_keeps_original() {
        let configurer = PreferencesPlaceholderConfigurer::new();
        let result = configurer.resolve_placeholders("${missing.key}");
        // Unresolved placeholder should remain as-is
        assert_eq!(result, "${missing.key}");
    }

    #[test]
    fn set_placeholder_prefix_and_suffix() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_placeholder_prefix("{{");
        configurer.set_placeholder_suffix("}}");
        configurer.set_preference("key", "value");
        let result = configurer.resolve_placeholders("{{key}}");
        assert_eq!(result, "value");
    }

    #[test]
    fn set_placeholder_prefix_string() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_placeholder_prefix("<<".to_string());
        configurer.set_placeholder_suffix(">>".to_string());
        configurer.set_preference("k", "v");
        let result = configurer.resolve_placeholders("<<k>>");
        assert_eq!(result, "v");
    }

    #[test]
    fn resolve_placeholders_multiple() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("a", "1");
        configurer.set_preference("b", "2");
        configurer.set_preference("c", "3");
        let result = configurer.resolve_placeholders("${a}-${b}-${c}");
        assert_eq!(result, "1-2-3");
    }

    #[test]
    fn debug_format() {
        let configurer = PreferencesPlaceholderConfigurer::new();
        let debug = format!("{:?}", configurer);
        assert!(debug.contains("PreferencesPlaceholderConfigurer"));
    }

    #[test]
    fn resolve_placeholder_with_string_args() {
        let mut configurer = PreferencesPlaceholderConfigurer::new();
        configurer.set_preference("key".to_string(), "value".to_string());
        assert_eq!(configurer.get_preference("key"), Some("value"));
    }
}
