//! PlaceholderConfigurerSupport — 对应 Spring `org.springframework.beans.factory.config.PlaceholderConfigurerSupport`。
//!
//! 占位符配置器支持基类。

use std::collections::HashMap;

/// 占位符配置器支持基类。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PlaceholderConfigurerSupport`。
///
/// 为占位符配置器提供基础功能，包括：
/// - 占位符前缀/后缀配置
/// - 值分隔符配置
/// - 占位符解析
///
/// ## 默认值
///
/// - 前缀：`${`
/// - 后缀：`}`
/// - 值分隔符：`:`（用于默认值）
#[derive(Debug)]
pub struct PlaceholderConfigurerSupport {
    /// 占位符前缀。
    placeholder_prefix: String,
    /// 占位符后缀。
    placeholder_suffix: String,
    /// 值分隔符（用于默认值）。
    value_separator: Option<String>,
    /// 是否忽略未解析的占位符。
    ignore_unresolvable_placeholders: bool,
}

impl PlaceholderConfigurerSupport {
    /// 创建新的 PlaceholderConfigurerSupport。
    pub fn new() -> Self {
        Self {
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
            value_separator: Some(":".to_string()),
            ignore_unresolvable_placeholders: false,
        }
    }

    /// 设置占位符前缀。
    pub fn set_placeholder_prefix(&mut self, prefix: impl Into<String>) {
        self.placeholder_prefix = prefix.into();
    }

    /// 获取占位符前缀。
    pub fn placeholder_prefix(&self) -> &str {
        &self.placeholder_prefix
    }

    /// 设置占位符后缀。
    pub fn set_placeholder_suffix(&mut self, suffix: impl Into<String>) {
        self.placeholder_suffix = suffix.into();
    }

    /// 获取占位符后缀。
    pub fn placeholder_suffix(&self) -> &str {
        &self.placeholder_suffix
    }

    /// 设置值分隔符。
    pub fn set_value_separator(&mut self, separator: impl Into<String>) {
        self.value_separator = Some(separator.into());
    }

    /// 获取值分隔符。
    pub fn value_separator(&self) -> Option<&str> {
        self.value_separator.as_deref()
    }

    /// 设置是否忽略未解析的占位符。
    pub fn set_ignore_unresolvable_placeholders(&mut self, ignore: bool) {
        self.ignore_unresolvable_placeholders = ignore;
    }

    /// 是否忽略未解析的占位符。
    pub fn ignore_unresolvable_placeholders(&self) -> bool {
        self.ignore_unresolvable_placeholders
    }

    /// 解析占位符。
    pub fn parse_placeholders(
        &self,
        text: &str,
        properties: &HashMap<String, String>,
    ) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find(&self.placeholder_prefix) {
            if let Some(end) = result[start..].find(&self.placeholder_suffix) {
                let placeholder_content =
                    &result[start + self.placeholder_prefix.len()..start + end];

                // 检查是否有默认值
                let (key, default_value) = if let Some(separator_pos) =
                    placeholder_content.find(self.value_separator.as_deref().unwrap_or(":"))
                {
                    (
                        &placeholder_content[..separator_pos],
                        Some(&placeholder_content[separator_pos + 1..]),
                    )
                } else {
                    (placeholder_content, None)
                };

                if let Some(resolved) = properties.get(key) {
                    result = format!(
                        "{}{}{}",
                        &result[..start],
                        resolved,
                        &result[start + end + self.placeholder_suffix.len()..]
                    );
                } else if let Some(default) = default_value {
                    result = format!(
                        "{}{}{}",
                        &result[..start],
                        default,
                        &result[start + end + self.placeholder_suffix.len()..]
                    );
                } else if self.ignore_unresolvable_placeholders {
                    break;
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

impl Default for PlaceholderConfigurerSupport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_configurer_support_defaults() {
        let configurer = PlaceholderConfigurerSupport::new();
        assert_eq!(configurer.placeholder_prefix(), "${");
        assert_eq!(configurer.placeholder_suffix(), "}");
        assert_eq!(configurer.value_separator(), Some(":"));
        assert!(!configurer.ignore_unresolvable_placeholders());
    }

    #[test]
    fn test_placeholder_configurer_support_parse() {
        let configurer = PlaceholderConfigurerSupport::new();
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), "Alice".to_string());

        let result = configurer.parse_placeholders("Hello ${name}!", &properties);
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_placeholder_configurer_support_default_value() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();

        let result = configurer.parse_placeholders("Hello ${name:World}!", &properties);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_default_trait() {
        let configurer = PlaceholderConfigurerSupport::default();
        assert_eq!(configurer.placeholder_prefix(), "${");
    }

    #[test]
    fn test_set_placeholder_prefix() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        configurer.set_placeholder_prefix("#{");
        assert_eq!(configurer.placeholder_prefix(), "#{");
    }

    #[test]
    fn test_set_placeholder_suffix() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        configurer.set_placeholder_suffix("}}");
        assert_eq!(configurer.placeholder_suffix(), "}}");
    }

    #[test]
    fn test_set_value_separator() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        configurer.set_value_separator("::");
        assert_eq!(configurer.value_separator(), Some("::"));
    }

    #[test]
    fn test_set_ignore_unresolvable() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        assert!(!configurer.ignore_unresolvable_placeholders());
        configurer.set_ignore_unresolvable_placeholders(true);
        assert!(configurer.ignore_unresolvable_placeholders());
    }

    #[test]
    fn test_no_placeholders() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("No placeholders here", &properties);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_multiple_placeholders() {
        let configurer = PlaceholderConfigurerSupport::new();
        let mut properties = HashMap::new();
        properties.insert("host".to_string(), "localhost".to_string());
        properties.insert("port".to_string(), "8080".to_string());

        let result = configurer.parse_placeholders("${host}:${port}", &properties);
        assert_eq!(result, "localhost:8080");
    }

    #[test]
    fn test_unresolvable_with_ignore() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        configurer.set_ignore_unresolvable_placeholders(true);
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("Hello ${name}!", &properties);
        // With ignore, unresolvable placeholders are left as-is
        assert_eq!(result, "Hello ${name}!");
    }

    #[test]
    fn test_unresolvable_without_ignore() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("Hello ${name}!", &properties);
        // Without ignore, unresolvable placeholders are left as-is (break)
        assert_eq!(result, "Hello ${name}!");
    }

    #[test]
    fn test_default_value_used_when_missing() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("${missing:fallback}", &properties);
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_property_overrides_default() {
        let configurer = PlaceholderConfigurerSupport::new();
        let mut properties = HashMap::new();
        properties.insert("key".to_string(), "actual".to_string());
        let result = configurer.parse_placeholders("${key:default}", &properties);
        assert_eq!(result, "actual");
    }

    #[test]
    fn test_custom_prefix_suffix() {
        let mut configurer = PlaceholderConfigurerSupport::new();
        configurer.set_placeholder_prefix("#{");
        configurer.set_placeholder_suffix("}");
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), "Alice".to_string());
        let result = configurer.parse_placeholders("Hello #{name}!", &properties);
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_empty_text() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("", &properties);
        assert_eq!(result, "");
    }

    #[test]
    fn test_malformed_placeholder_no_suffix() {
        let configurer = PlaceholderConfigurerSupport::new();
        let properties = HashMap::new();
        let result = configurer.parse_placeholders("Hello ${name", &properties);
        assert_eq!(result, "Hello ${name");
    }

    #[test]
    fn test_debug_format() {
        let configurer = PlaceholderConfigurerSupport::new();
        let debug = format!("{:?}", configurer);
        assert!(debug.contains("PlaceholderConfigurerSupport"));
    }
}
