//! 枚举 → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.EnumToStringConverter`。

use crate::convert::{ConversionError, Converter};

/// 枚举 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.EnumToStringConverter
///
/// Spring 语义：`Enum.name()`；Rust 中以 `Debug` 的变体名表达。
pub struct EnumToStringConverter;

impl<T: std::fmt::Debug> Converter<&T, String> for EnumToStringConverter {
    fn convert(&self, source: &T) -> Result<String, ConversionError> {
        Ok(format!("{source:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum Level {
        Low,
        High,
    }

    #[test]
    fn converts_variant_name() {
        // A 类（合同对齐）：对标 Spring `Enum.name()`
        let converter = EnumToStringConverter;
        assert_eq!(converter.convert(&Level::High).unwrap(), "High");
    }

    #[test]
    fn round_trip_via_from_str() {
        // D 类（重构安全）：与 `FromStr` 解析往返一致
        #[derive(Debug, PartialEq)]
        enum Mode {
            Prod,
            Dev,
        }
        impl std::str::FromStr for Mode {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "Prod" => Ok(Mode::Prod),
                    "Dev" => Ok(Mode::Dev),
                    _ => Err(format!("unknown: {s}")),
                }
            }
        }
        let converter = EnumToStringConverter;
        let name = converter.convert(&Mode::Dev).unwrap();
        assert_eq!(name.parse::<Mode>().unwrap(), Mode::Dev);
    }
}
