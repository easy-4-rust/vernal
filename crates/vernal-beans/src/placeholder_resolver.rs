//! PlaceholderResolver — `${...}` 占位符解析器。
//!
//! 对应 Java 类：`org.springframework.util.PropertyPlaceholderHandler` /
//! `org.springframework.core.env.PropertySourcesPropertyResolver` 的占位符解析部分。
//!
//! 从 `PropertySources` 中解析 `${...}` 形式的占位符。

use crate::property_sources::PropertySources;

/// 占位符前缀。
pub const PLACEHOLDER_PREFIX: &str = "${";
/// 占位符后缀。
pub const PLACEHOLDER_SUFFIX: &str = "}";
/// 值分隔符（用于 `${name:default}` 形式）。
pub const VALUE_SEPARATOR: &str = ":";

/// Spring 风格的占位符解析器。
///
/// 对应 Spring 的占位符解析逻辑。
///
/// 从 `PropertySources` 中按优先级解析 `${...}` 占位符。
/// 支持 `${name:default}` 形式提供默认值，支持嵌套占位符。
#[derive(Debug)]
pub struct PlaceholderResolver<'a> {
    property_sources: &'a PropertySources,
}

impl<'a> PlaceholderResolver<'a> {
    /// 创建占位符解析器。
    pub fn new(property_sources: &'a PropertySources) -> Self {
        Self { property_sources }
    }

    /// 解析字符串中的所有占位符。
    ///
    /// 遇到无法解析且无默认值的占位符时，原样保留该占位符。
    pub fn resolve_placeholders(&self, value: &str) -> String {
        self.resolve_internal(value, false)
    }

    /// 解析字符串中的所有占位符，遇到无法解析的占位符时返回 `Err`。
    pub fn resolve_required_placeholders(
        &self,
        value: &str,
    ) -> Result<String, UnresolvablePlaceholderError> {
        let unresolved = self.collect_unresolved(value);
        if unresolved.is_empty() {
            Ok(self.resolve_internal(value, false))
        } else {
            Err(UnresolvablePlaceholderError::new(
                value.to_owned(),
                unresolved,
            ))
        }
    }

    fn resolve_internal(&self, value: &str, _required: bool) -> String {
        let mut current = value.to_owned();
        // 限制最大迭代次数，防止递归占位符导致无限循环。
        for _ in 0..16 {
            if !current.contains(PLACEHOLDER_PREFIX) {
                break;
            }
            let next = self.replace_once_pass(&current);
            if next == current {
                break;
            }
            current = next;
        }
        current
    }

    /// 单轮替换：处理最外层占位符。
    fn replace_once_pass(&self, value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        let bytes = value.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if value[i..].starts_with(PLACEHOLDER_PREFIX) {
                if let Some((name, end)) = self.match_placeholder(value, i) {
                    let resolved = self.resolve_placeholder_name(&name);
                    out.push_str(&resolved);
                    i = end;
                    continue;
                }
            }
            // 安全地按字符边界推进
            let ch = value[i..].chars().next().expect("non-empty slice");
            out.push(ch);
            i += ch.len_utf8();
        }
        out
    }

    /// 匹配从 `start` 开始的占位符，返回占位符内部文本与结束位置。
    fn match_placeholder(&self, value: &str, start: usize) -> Option<(String, usize)> {
        let after_prefix = start + PLACEHOLDER_PREFIX.len();
        let mut depth = 1usize;
        let mut idx = after_prefix;
        let chars: Vec<(usize, char)> = value[after_prefix..].char_indices().collect();
        for (offset, ch) in chars {
            let abs = after_prefix + offset;
            if value[abs..].starts_with(PLACEHOLDER_PREFIX) {
                depth += 1;
            } else if value[abs..].starts_with(PLACEHOLDER_SUFFIX) {
                depth -= 1;
                if depth == 0 {
                    let inner = &value[after_prefix..abs];
                    return Some((inner.to_owned(), abs + PLACEHOLDER_SUFFIX.len()));
                }
            }
            idx = abs + ch.len_utf8();
        }
        let _ = idx;
        None
    }

    /// 解析单个占位符名（含默认值分隔）。
    fn resolve_placeholder_name(&self, raw_name: &str) -> String {
        // 处理 `name:default` 形式。
        let (name, default) = match raw_name.split_once(VALUE_SEPARATOR) {
            Some((n, d)) => (n.trim(), Some(d)),
            None => (raw_name.trim(), None),
        };
        if let Some(value) = self.property_sources.get_property(name) {
            return value.to_owned();
        }
        match default {
            Some(d) => {
                // 默认值自身可能含占位符，递归处理一轮。
                if d.contains(PLACEHOLDER_PREFIX) {
                    self.resolve_internal(d, false)
                } else {
                    d.to_owned()
                }
            }
            None => format!("{PLACEHOLDER_PREFIX}{raw_name}{PLACEHOLDER_SUFFIX}"),
        }
    }

    fn collect_unresolved(&self, value: &str) -> Vec<String> {
        let resolved = self.resolve_internal(value, false);
        let mut unresolved = Vec::new();
        let mut rest = resolved.as_str();
        while let Some(start) = rest.find(PLACEHOLDER_PREFIX) {
            if let Some(end_rel) = rest[start..].find(PLACEHOLDER_SUFFIX) {
                let end = start + end_rel + PLACEHOLDER_SUFFIX.len();
                unresolved.push(rest[start..end].to_owned());
                rest = &rest[end..];
            } else {
                break;
            }
        }
        unresolved
    }
}

/// 无法解析的占位符错误。
#[derive(Debug, Clone)]
pub struct UnresolvablePlaceholderError {
    /// 原始输入。
    pub input: String,
    /// 未能解析的占位符列表。
    pub placeholders: Vec<String>,
}

impl std::fmt::Display for UnresolvablePlaceholderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not resolve placeholder(s) {:?} in value \"{}\"",
            self.placeholders, self.input
        )
    }
}

impl std::error::Error for UnresolvablePlaceholderError {}

impl UnresolvablePlaceholderError {
    /// 创建错误。
    pub fn new(input: String, placeholders: Vec<String>) -> Self {
        Self {
            input,
            placeholders,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_source::PropertySource;

    fn build() -> PropertySources {
        let mut sources = PropertySources::new();
        sources.add(PropertySource::from_pairs(
            "app",
            [("name", "vernal"), ("greeting", "Hello ${name}!")],
        ));
        sources
    }

    #[test]
    fn test_simple_resolution() {
        let sources = build();
        let resolver = PlaceholderResolver::new(&sources);
        assert_eq!(resolver.resolve_placeholders("${name}"), "vernal");
    }

    #[test]
    fn test_nested_resolution() {
        let sources = build();
        let resolver = PlaceholderResolver::new(&sources);
        assert_eq!(
            resolver.resolve_placeholders("${greeting}"),
            "Hello vernal!"
        );
    }

    #[test]
    fn test_default_value() {
        let sources = build();
        let resolver = PlaceholderResolver::new(&sources);
        assert_eq!(
            resolver.resolve_placeholders("${missing:fallback}"),
            "fallback"
        );
    }

    #[test]
    fn test_required_unresolvable() {
        let sources = build();
        let resolver = PlaceholderResolver::new(&sources);
        let result = resolver.resolve_required_placeholders("${totally.unknown}");
        assert!(result.is_err());
    }
}
