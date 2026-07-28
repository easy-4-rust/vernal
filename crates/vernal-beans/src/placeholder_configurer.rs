//! PlaceholderConfigurerSupport — Spring 风格的占位符配置器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.PlaceholderConfigurerSupport`。
//!
//! 解析 Bean 定义中的 `${...}` 占位符，替换为实际值。

use std::collections::HashMap;
use std::sync::Arc;

/// Spring 风格的占位符配置器。
///
/// 对应 Spring 的 `PlaceholderConfigurerSupport`。
///
/// 解析 Bean 定义中的 `${...}` 占位符，替换为实际值。
/// 支持：
/// - `${property.name}` — 替换为属性值
/// - `${property.name:default}` — 带默认值的占位符
/// - 嵌套占位符
///
/// ## 与 vernal-context 的关系
///
/// `PlaceholderConfigurer` 可以从 `vernal-context::ApplicationEnvironment`
/// 加载属性源，实现 `@PropertySource` 风格的配置加载。
pub struct PlaceholderConfigurerSupport {
    /// 属性源（key → value）。
    properties: HashMap<String, String>,
    /// 占位符前缀。
    placeholder_prefix: String,
    /// 占位符后缀。
    placeholder_suffix: String,
    /// 值分隔符。
    value_separator: String,
}

impl PlaceholderConfigurerSupport {
    /// 创建占位符配置器。
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
            value_separator: ":".to_string(),
        }
    }

    /// 设置属性。
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// 批量设置属性。
    pub fn set_properties(&mut self, props: HashMap<String, String>) {
        self.properties.extend(props);
    }

    /// 获取属性。
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|v| v.as_str())
    }

    /// 获取所有属性。
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
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

    /// 解析字符串中的占位符。
    ///
    /// 对应 Spring 的 `PlaceholderConfigurerSupport.resolvePlaceholder(String, Properties)`。
    pub fn resolve_placeholder(&self, text: &str) -> String {
        let mut result = text.to_string();
        let prefix = &self.placeholder_prefix;
        let suffix = &self.placeholder_suffix;
        let separator = &self.value_separator;

        // 循环替换所有占位符
        while let Some(start) = result.find(prefix) {
            let after_prefix = &result[start + prefix.len()..];
            if let Some(end) = after_prefix.find(suffix) {
                let placeholder = &after_prefix[..end];
                // 解析占位符：key:default
                let (key, default) = if let Some(sep_pos) = placeholder.find(separator) {
                    let key = &placeholder[..sep_pos];
                    let default = &placeholder[sep_pos + separator.len()..];
                    (key, Some(default))
                } else {
                    (placeholder, None)
                };

                let replacement = if let Some(val) = self.properties.get(key) {
                    val.clone()
                } else if let Some(def) = default {
                    def.to_string()
                } else {
                    String::new()
                };

                let full_match = format!("{}{}{}", prefix, placeholder, suffix);
                result = result.replace(&full_match, &replacement);
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
