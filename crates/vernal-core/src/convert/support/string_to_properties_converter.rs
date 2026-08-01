//! 字符串 → Properties 转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToPropertiesConverter`。

use std::collections::BTreeMap;

use crate::convert::{ConversionError, Converter};
use crate::properties_file::parse_properties;

/// 字符串 → 属性表转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToPropertiesConverter
///
/// Spring 语义：把 `key=value` 文本解析为 `Properties`（注释以 `#`/`!` 开头，
/// 空行忽略）。Rust 中以 `BTreeMap<String, String>` 表达属性表。
pub struct StringToPropertiesConverter;

impl Converter<&str, BTreeMap<String, String>> for StringToPropertiesConverter {
    fn convert(&self, source: &str) -> Result<BTreeMap<String, String>, ConversionError> {
        let mut map = BTreeMap::new();
        let mut flat: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        parse_properties(source, &mut flat);
        map.extend(flat);
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_value_lines() {
        // A 类（合同对齐）：对标 Spring Properties 解析
        let converter = StringToPropertiesConverter;
        let map = converter.convert("a=1\nb=2\n").unwrap();
        assert_eq!(map.get("a").map(String::as_str), Some("1"));
        assert_eq!(map.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        // B 类（边界行为）：注释与空行不产生条目
        let converter = StringToPropertiesConverter;
        let map = converter.convert("# comment\n\n! another\nk=v\n").unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("k").map(String::as_str), Some("v"));
    }
}
