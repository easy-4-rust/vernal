//! 属性占位符解析器。
//!
//! 对标 Spring `org.springframework.util.PropertyPlaceholderHelper`。
//!
//! 解析字符串中的 `${...}` 占位符,用属性值替换。
//!
//! # 示例
//!
//! ```rust
//! use vernal_core::util::PropertyPlaceholderHelper;
//! use std::collections::HashMap;
//!
//! let mut props = HashMap::new();
//! props.insert("name".to_string(), "vernal".to_string());
//! props.insert("version".to_string(), "1.0".to_string());
//!
//! let helper = PropertyPlaceholderHelper::with_default();
//! let result = helper.replace_placeholders("Hello, ${name} v${version}!", |key| {
//!     props.get(key).cloned()
//! });
//! assert_eq!(result, "Hello, vernal v1.0!");
//! ```

use std::collections::HashMap;

/// 占位符解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceholderError {
    /// 占位符未解析(无默认值且属性源未提供)。
    UnresolvedPlaceholder {
        /// 未解析的占位符名称
        placeholder: String,
    },
    /// 占位符语法错误。
    InvalidSyntax {
        /// 原始字符串
        input: String,
        /// 错误原因
        reason: String,
    },
}

impl std::fmt::Display for PlaceholderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedPlaceholder { placeholder } => {
                write!(f, "Could not resolve placeholder '{placeholder}'")
            }
            Self::InvalidSyntax { input, reason } => {
                write!(f, "Invalid placeholder syntax in '{input}': {reason}")
            }
        }
    }
}

impl std::error::Error for PlaceholderError {}

/// 占位符解析函数 trait。
///
/// 对标 Spring `PropertyPlaceholderHelper.PlaceholderResolver` 函数式接口。
pub trait PlaceholderResolver: Send + Sync {
    /// 根据占位符名称解析属性值。
    ///
    /// 返回 `None` 表示未找到。
    fn resolve_placeholder(&self, placeholder_name: &str) -> Option<String>;
}

/// 为闭包实现 `PlaceholderResolver`。
impl<F> PlaceholderResolver for F
where
    F: Fn(&str) -> Option<String> + Send + Sync,
{
    fn resolve_placeholder(&self, placeholder_name: &str) -> Option<String> {
        (self)(placeholder_name)
    }
}

/// 为 `HashMap` 实现 `PlaceholderResolver`。
impl PlaceholderResolver for HashMap<String, String> {
    fn resolve_placeholder(&self, placeholder_name: &str) -> Option<String> {
        self.get(placeholder_name).cloned()
    }
}

/// 占位符前缀(默认 `${`)。
pub const DEFAULT_PLACEHOLDER_PREFIX: &str = "${";
/// 占位符后缀(默认 `}`)。
pub const DEFAULT_PLACEHOLDER_SUFFIX: &str = "}";
/// 默认值分隔符(默认 `:`)。
pub const DEFAULT_VALUE_SEPARATOR: &str = ":";

/// 属性占位符解析器。
///
/// 对标 Spring `PropertyPlaceholderHelper`。
#[derive(Debug, Clone)]
pub struct PropertyPlaceholderHelper {
    /// 占位符前缀(如 `${`)。
    placeholder_prefix: String,
    /// 占位符后缀(如 `}`)。
    placeholder_suffix: String,
    /// 默认值分隔符(如 `:`)。
    value_separator: String,
    /// 是否忽略未解析的占位符(默认 false,对标 Spring `ignoreUnresolvablePlaceholders`)。
    ignore_unresolvable: bool,
}

impl PropertyPlaceholderHelper {
    /// 使用默认配置(`${...}` 前缀,`:` 分隔符,严格模式)创建。
    #[must_use]
    pub fn with_default() -> Self {
        Self {
            placeholder_prefix: DEFAULT_PLACEHOLDER_PREFIX.to_string(),
            placeholder_suffix: DEFAULT_PLACEHOLDER_SUFFIX.to_string(),
            value_separator: DEFAULT_VALUE_SEPARATOR.to_string(),
            ignore_unresolvable: false,
        }
    }

    /// 使用自定义前缀/后缀/分隔符创建。
    ///
    /// 对标 Spring 构造器 `PropertyPlaceholderHelper(String, String, String, ...)`。
    #[must_use]
    pub fn new(
        placeholder_prefix: impl Into<String>,
        placeholder_suffix: impl Into<String>,
        value_separator: impl Into<String>,
    ) -> Self {
        Self {
            placeholder_prefix: placeholder_prefix.into(),
            placeholder_suffix: placeholder_suffix.into(),
            value_separator: value_separator.into(),
            ignore_unresolvable: false,
        }
    }

    /// 设置是否忽略未解析的占位符。
    ///
    /// 对标 Spring `setIgnoreUnresolvablePlaceholders(boolean)`。
    pub fn set_ignore_unresolvable(&mut self, ignore: bool) {
        self.ignore_unresolvable = ignore;
    }

    /// 替换字符串中所有占位符。
    ///
    /// 对标 Spring `replacePlaceholders(String, PlaceholderResolver)`。
    ///
    /// # 错误
    ///
    /// 当 `ignore_unresolvable = false` 且存在未解析的占位符时返回 `Err`。
    pub fn replace_placeholders<R: PlaceholderResolver>(
        &self,
        value: &str,
        resolver: R,
    ) -> Result<String, PlaceholderError> {
        self.parse_string(value, &resolver, &mut Vec::new())
    }

    /// 替换字符串中所有占位符(忽略错误,未解析的保持原样)。
    ///
    /// 对标 Spring `replacePlaceholders` 在 `ignoreUnresolvablePlaceholders=true` 模式下的行为。
    pub fn replace_placeholders_lenient<R: PlaceholderResolver>(
        &self,
        value: &str,
        resolver: R,
    ) -> String {
        self.replace_placeholders(value, resolver)
            .unwrap_or_else(|_| value.to_string())
    }

    fn parse_string<R: PlaceholderResolver>(
        &self,
        value: &str,
        resolver: &R,
        visited: &mut Vec<String>,
    ) -> Result<String, PlaceholderError> {
        let prefix = self.placeholder_prefix.as_str();
        let suffix = self.placeholder_suffix.as_str();
        let prefix_len = prefix.len();

        let mut result = String::with_capacity(value.len());
        let mut i = 0;
        let byte_len = value.len();

        while i < byte_len {
            // 检查当前位置是否匹配前缀
            if byte_len - i >= prefix_len && value[i..].starts_with(prefix) {
                // 找到配对的 suffix(考虑嵌套)
                match self.find_matching_suffix(value, i + prefix_len) {
                    Some(end_idx) => {
                        let placeholder_content = &value[i + prefix_len..end_idx];
                        let resolved = self.resolve_placeholder_content(
                            placeholder_content,
                            resolver,
                            visited,
                        )?;
                        result.push_str(&resolved);
                        i = end_idx + suffix.len();
                    }
                    None => {
                        return Err(PlaceholderError::InvalidSyntax {
                            input: value.to_string(),
                            reason: format!("missing closing suffix '{suffix}' after position {i}"),
                        });
                    }
                }
            } else {
                // 普通字符:复制一个字节(ASCII)或一个字符(多字节)
                let ch = value[i..].chars().next().unwrap_or_default();
                result.push(ch);
                i += ch.len_utf8();
            }
        }

        Ok(result)
    }

    /// 寻找匹配的后缀(考虑嵌套深度)。
    fn find_matching_suffix(&self, value: &str, start: usize) -> Option<usize> {
        let prefix = self.placeholder_prefix.as_str();
        let suffix = self.placeholder_suffix.as_str();
        let prefix_len = prefix.len();
        let suffix_len = suffix.len();
        let byte_len = value.len();

        let mut depth: i32 = 1;
        let mut i = start;
        while i < byte_len {
            if byte_len - i >= prefix_len && value[i..].starts_with(prefix) {
                depth += 1;
                i += prefix_len;
            } else if byte_len - i >= suffix_len && value[i..].starts_with(suffix) {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += suffix_len;
            } else {
                i += 1;
            }
        }
        None
    }

    fn resolve_placeholder_content<R: PlaceholderResolver>(
        &self,
        content: &str,
        resolver: &R,
        visited: &mut Vec<String>,
    ) -> Result<String, PlaceholderError> {
        // 检查循环引用
        if visited.contains(&content.to_string()) {
            return Err(PlaceholderError::InvalidSyntax {
                input: content.to_string(),
                reason: format!("circular reference: {visited:?}"),
            });
        }

        // 处理默认值分隔符(${key:default})
        let (key, default_value) = match content.find(self.value_separator.as_str()) {
            Some(idx) => {
                let k = &content[..idx];
                let v = &content[idx + self.value_separator.len()..];
                (k, Some(v.to_string()))
            }
            None => (content, None),
        };

        // 递归解析 key 本身可能包含的占位符
        let resolved_key = if key.contains(self.placeholder_prefix.as_str()) {
            self.parse_string(key, resolver, visited)?
        } else {
            key.to_string()
        };

        visited.push(resolved_key.clone());

        let result = match resolver.resolve_placeholder(&resolved_key) {
            Some(val) => {
                // 解析值中可能包含的占位符(嵌套)
                if val.contains(self.placeholder_prefix.as_str()) {
                    self.parse_string(&val, resolver, visited)
                } else {
                    Ok(val)
                }
            }
            None => match default_value {
                Some(v) => Ok(v),
                None => {
                    if self.ignore_unresolvable {
                        // 保留原始占位符
                        Ok(format!(
                            "{}{}{}",
                            self.placeholder_prefix, content, self.placeholder_suffix
                        ))
                    } else {
                        Err(PlaceholderError::UnresolvedPlaceholder {
                            placeholder: resolved_key.clone(),
                        })
                    }
                }
            },
        };

        visited.pop();
        result
    }
}

impl Default for PropertyPlaceholderHelper {
    fn default() -> Self {
        Self::with_default()
    }
}

/// 简单字符串值解析器(对标 Spring `StringValueResolver`)。
///
/// 用于把字符串值(可能是占位符)解析为最终值。
pub trait StringValueResolver: Send + Sync {
    /// 解析字符串值。
    fn resolve_string_value(&self, str_val: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver_from<'a>(map: &'a [(&'a str, &'a str)]) -> impl PlaceholderResolver + 'a {
        move |key: &str| -> Option<String> {
            map.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn replace_single_placeholder() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders("Hello, ${name}!", resolver_from(&[("name", "world")]))
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn replace_multiple_placeholders() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders(
                "${greeting}, ${name}!",
                resolver_from(&[("greeting", "Hello"), ("name", "world")]),
            )
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn replace_with_default_value() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders("Hello, ${name:default}!", resolver_from(&[]))
            .unwrap();
        assert_eq!(result, "Hello, default!");
    }

    #[test]
    fn replace_default_value_overridden_by_actual_value() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders(
                "Hello, ${name:default}!",
                resolver_from(&[("name", "actual")]),
            )
            .unwrap();
        assert_eq!(result, "Hello, actual!");
    }

    #[test]
    fn unresolved_placeholder_returns_error_by_default() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper.replace_placeholders("${missing}", resolver_from(&[]));
        assert!(matches!(
            result,
            Err(PlaceholderError::UnresolvedPlaceholder { .. })
        ));
    }

    #[test]
    fn ignore_unresolvable_keeps_original() {
        let mut helper = PropertyPlaceholderHelper::with_default();
        helper.set_ignore_unresolvable(true);
        let result = helper
            .replace_placeholders("${missing}", resolver_from(&[]))
            .unwrap();
        assert_eq!(result, "${missing}");
    }

    #[test]
    fn no_placeholders_returned_as_is() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders("plain text", resolver_from(&[]))
            .unwrap();
        assert_eq!(result, "plain text");
    }

    #[test]
    fn empty_string_returns_empty() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper.replace_placeholders("", resolver_from(&[])).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn nested_placeholders_resolved() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders(
                "${${key}}",
                resolver_from(&[("key", "name"), ("name", "vernal")]),
            )
            .unwrap();
        assert_eq!(result, "vernal");
    }

    #[test]
    fn placeholder_in_default_value_is_literal_in_spring_semantics() {
        // 注意:Spring PropertyPlaceholderHelper 默认不递归解析默认值中的占位符
        // 默认值 ${missing:${fallback}} 中,${fallback} 被作为字面字符串保留
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders(
                "${missing:${fallback}}",
                resolver_from(&[("fallback", "default-value")]),
            )
            .unwrap();
        // Spring 保留 ${fallback} 作为字面默认值
        assert_eq!(result, "${fallback}");
    }

    #[test]
    fn hashmap_resolver_works() {
        let helper = PropertyPlaceholderHelper::with_default();
        let mut map = HashMap::new();
        map.insert("name".to_string(), "vernal".to_string());

        let result = helper.replace_placeholders("Hello, ${name}!", map).unwrap();
        assert_eq!(result, "Hello, vernal!");
    }

    #[test]
    fn replace_placeholders_lenient_returns_original_on_error() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper.replace_placeholders_lenient("${missing}", resolver_from(&[]));
        assert_eq!(result, "${missing}");
    }

    #[test]
    fn invalid_syntax_missing_closing_suffix() {
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper.replace_placeholders("${unclosed", resolver_from(&[]));
        assert!(matches!(
            result,
            Err(PlaceholderError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn custom_prefix_suffix_separator() {
        let helper = PropertyPlaceholderHelper::new("<%", "%>", "=");
        let result = helper
            .replace_placeholders("Hello, <%name%>!", resolver_from(&[("name", "world")]))
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn custom_separator_for_default() {
        let helper = PropertyPlaceholderHelper::new("${", "}", "|");
        let result = helper
            .replace_placeholders("${name|default}", resolver_from(&[]))
            .unwrap();
        assert_eq!(result, "default");
    }

    #[test]
    fn with_default_uses_colon_separator() {
        // 对标 Spring: 默认分隔符为 ":"
        let helper = PropertyPlaceholderHelper::with_default();
        let result = helper
            .replace_placeholders("${missing:fallback}", resolver_from(&[]))
            .unwrap();
        assert_eq!(result, "fallback");
    }

    #[test]
    fn circular_reference_detected() {
        let helper = PropertyPlaceholderHelper::with_default();
        // a 引用 b,b 引用 a
        let result =
            helper.replace_placeholders("${a}", resolver_from(&[("a", "${b}"), ("b", "${a}")]));
        assert!(matches!(
            result,
            Err(PlaceholderError::InvalidSyntax { .. })
        ));
    }

    #[test]
    fn placeholder_error_display() {
        let err = PlaceholderError::UnresolvedPlaceholder {
            placeholder: "missing".to_string(),
        };
        assert!(err.to_string().contains("missing"));

        let err = PlaceholderError::InvalidSyntax {
            input: "bad".to_string(),
            reason: "test".to_string(),
        };
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn default_trait_impl_same_as_with_default() {
        // 对标 Spring: new PropertyPlaceholderHelper() 使用默认前缀/后缀/分隔符
        // 覆盖行 310-312: Default trait 实现
        let helper = PropertyPlaceholderHelper::default();
        let result = helper
            .replace_placeholders("${key}", resolver_from(&[("key", "value")]))
            .unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn placeholder_error_is_std_error() {
        // 对标 Spring: PlaceholderError 应实现 std::error::Error
        fn assert_error<T: std::error::Error>() {}
        assert_error::<PlaceholderError>();
    }
}
