//! 流转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StreamConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 流转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StreamConverter
///
/// Spring 语义：`Stream<T>` 与集合/数组之间转换；Rust 中以 `Vec` 表达流
/// 的物化结果，支持集合 → 集合的逐元素转换。
pub struct StreamConverter;

impl<S: ToString, T: Convertible> Converter<&[S], Vec<T>> for StreamConverter {
    fn convert(&self, source: &[S]) -> Result<Vec<T>, ConversionError> {
        source
            .iter()
            .map(|item| T::from_str_value(&item.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_stream_elements() {
        // A 类（合同对齐）：对标 Spring Stream 元素转换
        let converter = StreamConverter;
        let output: Vec<String> = converter.convert(&[1_i64, 2]).unwrap();
        assert_eq!(output, vec!["1".to_string(), "2".to_string()]);
    }

    #[test]
    fn preserves_order() {
        // B 类（边界行为）：顺序保持
        let converter = StreamConverter;
        let output: Vec<String> = converter.convert(&["c", "a", "b"]).unwrap();
        assert_eq!(output, vec!["c", "a", "b"]);
    }
}
