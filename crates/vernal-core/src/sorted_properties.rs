//! 有序属性集合。
//!
//! 对标 Spring `org.springframework.core.SortedProperties`。
//!
//! Spring `SortedProperties` 继承自 `Properties`,按 key 自然顺序输出。
//! vernal-core 用 `BTreeMap<String, String>` 实现等价语义(天然有序)。

use std::collections::BTreeMap;
use std::fmt;

/// 有序属性集合。
///
/// 对应 Java: org.springframework.core.SortedProperties
/// 对标 Spring `SortedProperties`。按 key 字典序存储,用于配置文件输出
/// (保证每次生成的文件内容一致,便于 diff)。
///
/// # 示例
///
/// ```rust
/// use vernal_core::sorted_properties::SortedProperties;
///
/// let mut props = SortedProperties::new();
/// props.set("zulu", "3");
/// props.set("alpha", "1");
/// props.set("mike", "2");
///
/// // keys() 按字典序输出
/// let keys: Vec<&str> = props.keys().collect();
/// assert_eq!(keys, vec!["alpha", "mike", "zulu"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortedProperties {
    inner: BTreeMap<String, String>,
}

impl SortedProperties {
    /// 创建空的属性集合。
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// 设置属性(key 存在则覆盖)。
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    /// 获取属性值。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(String::as_str)
    }

    /// 检查属性是否存在。
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// 获取属性值,不存在则返回默认值。
    #[must_use]
    pub fn get_or_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// 移除属性。
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }

    /// 清空。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 属性数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 按字典序获取所有 key。
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.inner.keys().map(String::as_str)
    }

    /// 按字典序获取所有 value。
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.inner.values().map(String::as_str)
    }

    /// 按字典序获取所有 (key, value) 对。
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// 从 properties 格式字符串解析(每行 `key=value`,忽略 `#` 注释行)。
    ///
    /// 对标 Java `Properties.load(Reader)` 的简化版。
    #[must_use]
    pub fn parse_from_str(s: &str) -> Self {
        let mut props = Self::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                props.set(key.trim(), value.trim());
            }
        }
        props
    }

    /// 转换为 properties 格式字符串(按 key 字典序,`key=value` 格式)。
    ///
    /// 对标 Java `Properties.store()` 的简化版。
    #[must_use]
    pub fn to_properties_string(&self) -> String {
        self.inner
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for SortedProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SortedProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.inner {
            writeln!(f, "{k}={v}")?;
        }
        Ok(())
    }
}

impl FromIterator<(String, String)> for SortedProperties {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut props = Self::new();
        for (k, v) in iter {
            props.set(k, v);
        }
        props
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for SortedProperties {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut props = Self::new();
        for (k, v) in iter {
            props.set(k, v);
        }
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_sorted() {
        let mut p = SortedProperties::new();
        p.set("z", "3");
        p.set("a", "1");
        p.set("m", "2");
        let keys: Vec<&str> = p.keys().collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    #[test]
    fn set_and_get_basic() {
        let mut p = SortedProperties::new();
        p.set("key", "value");
        assert_eq!(p.get("key"), Some("value"));
        assert_eq!(p.get("missing"), None);
    }

    #[test]
    fn set_overwrites_previous() {
        let mut p = SortedProperties::new();
        p.set("key", "v1");
        p.set("key", "v2");
        assert_eq!(p.get("key"), Some("v2"));
    }

    #[test]
    fn get_or_default_basic() {
        let mut p = SortedProperties::new();
        p.set("key", "value");
        assert_eq!(p.get_or_default("key", "fallback"), "value");
        assert_eq!(p.get_or_default("missing", "fallback"), "fallback");
    }

    #[test]
    fn contains_key_basic() {
        let mut p = SortedProperties::new();
        p.set("a", "1");
        assert!(p.contains_key("a"));
        assert!(!p.contains_key("b"));
    }

    #[test]
    fn remove_basic() {
        let mut p = SortedProperties::new();
        p.set("a", "1");
        let removed = p.remove("a");
        assert_eq!(removed, Some("1".to_string()));
        assert!(p.get("a").is_none());
    }

    #[test]
    fn len_and_is_empty() {
        let mut p = SortedProperties::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        p.set("a", "1");
        assert_eq!(p.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn clear_empties_map() {
        let mut p = SortedProperties::new();
        p.set("a", "1");
        p.set("b", "2");
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn parse_from_str_basic() {
        let input = "alpha=1\nbeta=2\ngamma=3";
        let p = SortedProperties::parse_from_str(input);
        assert_eq!(p.len(), 3);
        assert_eq!(p.get("alpha"), Some("1"));
        assert_eq!(p.get("beta"), Some("2"));
        assert_eq!(p.get("gamma"), Some("3"));
    }

    #[test]
    fn parse_from_str_ignores_comments() {
        let input = "# comment\nalpha=1\n# another\nbeta=2";
        let p = SortedProperties::parse_from_str(input);
        assert_eq!(p.len(), 2);
        assert!(!p.contains_key("# comment"));
    }

    #[test]
    fn parse_from_str_ignores_empty_lines() {
        let input = "\n\nalpha=1\n\nbeta=2\n\n";
        let p = SortedProperties::parse_from_str(input);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn parse_from_str_handles_spaces() {
        let input = "  alpha = 1  \n  beta = 2  ";
        let p = SortedProperties::parse_from_str(input);
        assert_eq!(p.get("alpha"), Some("1"));
        assert_eq!(p.get("beta"), Some("2"));
    }

    #[test]
    fn to_properties_string_sorted() {
        let mut p = SortedProperties::new();
        p.set("z", "3");
        p.set("a", "1");
        p.set("m", "2");
        let s = p.to_properties_string();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines, vec!["a=1", "m=2", "z=3"]);
    }

    #[test]
    fn to_properties_string_round_trip() {
        let mut p = SortedProperties::new();
        p.set("key1", "value1");
        p.set("key2", "value2");
        let s = p.to_properties_string();
        let p2 = SortedProperties::parse_from_str(&s);
        assert_eq!(p.len(), p2.len());
        assert_eq!(p.get("key1"), p2.get("key1"));
        assert_eq!(p.get("key2"), p2.get("key2"));
    }

    #[test]
    fn values_returns_all_values() {
        let mut p = SortedProperties::new();
        p.set("a", "1");
        p.set("b", "2");
        let values: Vec<&str> = p.values().collect();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn entries_returns_key_value_pairs() {
        let mut p = SortedProperties::new();
        p.set("a", "1");
        p.set("b", "2");
        let entries: Vec<(&str, &str)> = p.entries().collect();
        assert_eq!(entries, vec![("a", "1"), ("b", "2")]);
    }

    #[test]
    fn display_outputs_properties_format() {
        let mut p = SortedProperties::new();
        p.set("key", "value");
        let s = format!("{p}");
        assert!(s.contains("key=value"));
    }

    #[test]
    fn from_iterator_string_pairs() {
        let p: SortedProperties = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn from_iterator_str_pairs() {
        let p: SortedProperties = vec![("a", "1"), ("b", "2")].into_iter().collect();
        assert_eq!(p.len(), 2);
        assert_eq!(p.get("a"), Some("1"));
    }

    #[test]
    fn default_is_empty() {
        let p = SortedProperties::default();
        assert!(p.is_empty());
    }
}
