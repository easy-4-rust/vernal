//! PropertyPlaceholderConfigurer — 对应 Spring `org.springframework.beans.factory.config.PropertyPlaceholderConfigurer`。
//!
//! 属性占位符配置器。

use std::collections::HashMap;

/// 属性占位符配置器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.PropertyPlaceholderConfigurer`。
///
/// 用于解析 Bean 定义中的 `${...}` 占位符。
/// 从属性文件、系统属性或环境变量中获取值。
///
/// ## 使用场景
///
/// - 解析配置文件中的占位符
/// - 支持多环境配置
/// - 属性覆盖和默认值
#[derive(Debug)]
pub struct PropertyPlaceholderConfigurer {
    /// 属性集合。
    properties: HashMap<String, String>,
    /// 占位符前缀。
    placeholder_prefix: String,
    /// 占位符后缀。
    placeholder_suffix: String,
    /// 值分隔符（用于默认值）。
    value_separator: String,
    /// 是否忽略未解析的占位符。
    ignore_unresolvable: bool,
    /// 系统属性模式。
    system_properties_mode: SystemPropertiesMode,
}

/// 系统属性模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPropertiesMode {
    /// 从不使用系统属性。
    Never,
    /// 在属性文件之后使用系统属性（覆盖）。
    Fallback,
    /// 在属性文件之前使用系统属性（优先）。
    Override,
}

impl Default for SystemPropertiesMode {
    fn default() -> Self {
        Self::Fallback
    }
}

impl PropertyPlaceholderConfigurer {
    /// 创建新的 PropertyPlaceholderConfigurer。
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
            value_separator: ":".to_string(),
            ignore_unresolvable: false,
            system_properties_mode: SystemPropertiesMode::default(),
        }
    }

    /// 设置属性。
    pub fn set_properties(&mut self, properties: HashMap<String, String>) {
        self.properties = properties;
    }

    /// 添加属性。
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// 获取属性值。
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    /// 设置占位符前缀。
    pub fn set_placeholder_prefix(&mut self, prefix: impl Into<String>) {
        self.placeholder_prefix = prefix.into();
    }

    /// 设置占位符后缀。
    pub fn set_placeholder_suffix(&mut self, suffix: impl Into<String>) {
        self.placeholder_suffix = suffix.into();
    }

    /// 设置值分隔符。
    pub fn set_value_separator(&mut self, separator: impl Into<String>) {
        self.value_separator = separator.into();
    }

    /// 设置是否忽略未解析的占位符。
    pub fn set_ignore_unresolvable(&mut self, ignore: bool) {
        self.ignore_unresolvable = ignore;
    }

    /// 设置系统属性模式。
    pub fn set_system_properties_mode(&mut self, mode: SystemPropertiesMode) {
        self.system_properties_mode = mode;
    }

    /// 解析占位符。
    pub fn resolve_placeholders(&self, text: &str) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find(&self.placeholder_prefix) {
            if let Some(end) = result[start..].find(&self.placeholder_suffix) {
                let placeholder_content =
                    &result[start + self.placeholder_prefix.len()..start + end];

                // 检查是否有默认值
                let (key, default_value) =
                    if let Some(separator_pos) = placeholder_content.find(&self.value_separator) {
                        (
                            &placeholder_content[..separator_pos],
                            Some(&placeholder_content[separator_pos + self.value_separator.len()..]),
                        )
                    } else {
                        (placeholder_content, None)
                    };

                if let Some(resolved) = self.properties.get(key) {
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
                } else if self.ignore_unresolvable {
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

impl Default for PropertyPlaceholderConfigurer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_placeholder_configurer_new() {
        let configurer = PropertyPlaceholderConfigurer::new();
        assert!(configurer.properties.get("key").is_none());
        assert!(!configurer.ignore_unresolvable);
    }

    #[test]
    fn test_property_placeholder_configurer_resolve() {
        let mut configurer = PropertyPlaceholderConfigurer::new();
        configurer.set_property("db.url", "jdbc:mysql://localhost:3306/mydb");

        let result = configurer.resolve_placeholders("URL: ${db.url}");
        assert_eq!(result, "URL: jdbc:mysql://localhost:3306/mydb");
    }

    #[test]
    fn test_property_placeholder_configurer_default_value() {
        let configurer = PropertyPlaceholderConfigurer::new();
        let result = configurer.resolve_placeholders("Port: ${port:8080}");
        assert_eq!(result, "Port: 8080");
    }
}
