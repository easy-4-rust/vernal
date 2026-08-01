//! 集合 → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.CollectionToStringConverter`。

use crate::convert::{ConversionError, Converter};

/// 集合 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.CollectionToStringConverter
///
/// Spring 语义：把集合元素逐个转换后以逗号拼接（对标
/// `StringUtils.collectionToCommaDelimitedString`）。
pub struct CollectionToStringConverter;

impl<T: std::fmt::Display> Converter<&[T], String> for CollectionToStringConverter {
    fn convert(&self, source: &[T]) -> Result<String, ConversionError> {
        let parts: Vec<String> = source.iter().map(ToString::to_string).collect();
        Ok(parts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_elements_with_comma() {
        // A 类（合同对齐）：对标 Spring 集合 → 逗号分隔字符串
        let converter = CollectionToStringConverter;
        let joined: String = converter.convert(&[1_i32, 2, 3]).unwrap();
        assert_eq!(joined, "1,2,3");
    }

    #[test]
    fn empty_collection_yields_empty_string() {
        // B 类（边界行为）
        let converter = CollectionToStringConverter;
        let joined: String = converter.convert(&Vec::<i32>::new()).unwrap();
        assert_eq!(joined, "");
    }

    #[test]
    fn round_trip_with_string_to_collection() {
        // D 类（重构安全）：与 `StringToCollectionConverter` 往返一致
        let to_string = CollectionToStringConverter;
        let from_string = crate::convert::support::StringToCollectionConverter;
        let joined: String = to_string.convert(&[1_i32, 2, 3]).unwrap();
        let parsed: Vec<i32> = from_string.convert(&joined).unwrap();
        assert_eq!(parsed, vec![1_i32, 2, 3]);
    }
}
