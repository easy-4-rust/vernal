//! 对象 → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ObjectToStringConverter`。

use crate::convert::{ConversionError, Converter};

/// 对象 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ObjectToStringConverter
///
/// Spring 语义：`Object.toString()`；Rust 中以 `Display` 表达。
pub struct ObjectToStringConverter;

impl<T: std::fmt::Display> Converter<&T, String> for ObjectToStringConverter {
    fn convert(&self, source: &T) -> Result<String, ConversionError> {
        Ok(source.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_display_value() {
        // A 类（合同对齐）：对标 Spring `Object.toString()`
        let converter = ObjectToStringConverter;
        assert_eq!(converter.convert(&42).unwrap(), "42");
        assert_eq!(converter.convert(&"hello").unwrap(), "hello");
    }

    #[test]
    fn converts_custom_display_type() {
        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }
        impl std::fmt::Display for Point {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}, {})", self.x, self.y)
            }
        }

        let converter = ObjectToStringConverter;
        assert_eq!(converter.convert(&Point { x: 1, y: 2 }).unwrap(), "(1, 2)");
    }
}
