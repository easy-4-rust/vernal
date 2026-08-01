//! Properties → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.PropertiesToStringConverter`。

use std::collections::BTreeMap;

use crate::convert::{ConversionError, Converter};

/// 属性表 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.PropertiesToStringConverter
///
/// Spring 语义：把 `Properties` 序列化为 `key=value` 文本（按 key 排序，
/// 对标 `Properties.store` 的确定性输出）。
pub struct PropertiesToStringConverter;

impl Converter<&BTreeMap<String, String>, String> for PropertiesToStringConverter {
    fn convert(&self, source: &BTreeMap<String, String>) -> Result<String, ConversionError> {
        let mut out = String::new();
        for (key, value) in source {
            out.push_str(key);
            out.push('=');
            out.push_str(value);
            out.push('\n');
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_in_sorted_order() {
        // A 类（合同对齐）：对标 Spring Properties.store 排序输出
        let converter = PropertiesToStringConverter;
        let mut map = BTreeMap::new();
        map.insert("b".to_string(), "2".to_string());
        map.insert("a".to_string(), "1".to_string());
        assert_eq!(converter.convert(&map).unwrap(), "a=1\nb=2\n");
    }

    #[test]
    fn round_trip_with_string_to_properties() {
        // D 类（重构安全）：与 `StringToPropertiesConverter` 往返一致
        let to_string = PropertiesToStringConverter;
        let from_string = crate::convert::support::StringToPropertiesConverter;
        let mut map = BTreeMap::new();
        map.insert("k".to_string(), "v".to_string());
        let text = to_string.convert(&map).unwrap();
        assert_eq!(from_string.convert(&text).unwrap(), map);
    }
}
