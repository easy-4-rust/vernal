//! .properties 格式解析工具。
//!
//! 提供 `key=value` 文本解析等纯工具函数，供属性加载、属性源工厂等复用；
//! Spring `SpringProperties` 门面语义由 [`crate::vernal_properties::VernalProperties`]
//! 承担。

use std::collections::HashMap;

/// 解析 .properties 格式文本。
///
/// 格式：`key=value` 每行一对，`#`/`!` 开头为注释。
#[allow(clippy::implicit_hasher)] // 保持签名与调用点简洁,不强制指定 hasher
pub fn parse_properties(content: &str, map: &mut HashMap<String, String>) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_properties_basic() {
        // A 类（合同对齐）：注释与键值对解析
        let mut map = HashMap::new();
        parse_properties(
            "key1=value1
# comment
key2=value2

! c2
k3=v3",
            &mut map,
        );
        assert_eq!(map.get("key1").map(String::as_str), Some("value1"));
        assert_eq!(map.get("key2").map(String::as_str), Some("value2"));
        assert_eq!(map.get("k3").map(String::as_str), Some("v3"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn parse_properties_trim() {
        // B 类（边界行为）：键值两侧空白被裁剪
        let mut map = HashMap::new();
        parse_properties("  key  =  value  ", &mut map);
        assert_eq!(map.get("key").map(String::as_str), Some("value"));
    }

    #[test]
    fn parse_properties_ignores_empty_and_comment_lines() {
        // B 类（边界行为）：空行与注释行不产生条目
        let mut map = HashMap::new();
        parse_properties("\n# c\n! c2\n", &mut map);
        assert!(map.is_empty());
    }

    #[test]
    fn parse_properties_line_without_equals_ignored() {
        // B 类（边界行为）：无 `=` 的行被忽略
        let mut map = HashMap::new();
        parse_properties("no-equals-here", &mut map);
        assert!(map.is_empty());
    }
}
