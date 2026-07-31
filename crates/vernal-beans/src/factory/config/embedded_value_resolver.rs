//! EmbeddedValueResolver — 对应 Spring `org.springframework.beans.factory.config.EmbeddedValueResolver`。
//!
//! 嵌入值解析器。

/// 嵌入值解析器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.EmbeddedValueResolver`。
///
/// 解析嵌入在字符串中的值（如 `${...}` 占位符）。
pub struct EmbeddedValueResolver {
    placeholder_prefix: String,
    placeholder_suffix: String,
}

impl EmbeddedValueResolver {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            placeholder_prefix: "${".to_string(),
            placeholder_suffix: "}".to_string(),
        }
    }

    /// 检查字符串是否包含占位符。
    pub fn contains_placeholder(&self, value: &str) -> bool {
        value.contains(&self.placeholder_prefix) && value.contains(&self.placeholder_suffix)
    }

    /// 解析字符串中的占位符。
    pub fn resolve(&self, value: &str, properties: &std::collections::HashMap<String, String>) -> String {
        let mut result = value.to_string();
        while let Some(start) = result.find(&self.placeholder_prefix) {
            if let Some(end) = result[start..].find(&self.placeholder_suffix) {
                let key = &result[start + self.placeholder_prefix.len()..start + end];
                if let Some(resolved) = properties.get(key) {
                    result = format!("{}{}{}", &result[..start], resolved, &result[start + end + self.placeholder_suffix.len()..]);
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

impl Default for EmbeddedValueResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_contains_placeholder() {
        let resolver = EmbeddedValueResolver::new();
        assert!(resolver.contains_placeholder("${key}"));
        assert!(!resolver.contains_placeholder("plain text"));
    }

    #[test]
    fn test_resolve() {
        let resolver = EmbeddedValueResolver::new();
        let mut props = HashMap::new();
        props.insert("name".to_string(), "Alice".to_string());

        let result = resolver.resolve("Hello ${name}!", &props);
        assert_eq!(result, "Hello Alice!");
    }
}
