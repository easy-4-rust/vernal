//! 条件枚举转换器抽象基类。
//!
//! 对标 Spring `org.springframework.core.convert.support.AbstractConditionalEnumConverter`。

use crate::convert::ConversionError;

/// 条件枚举转换器抽象（工厂内共享的枚举信息载体）。
///
/// 对应 Java: org.springframework.core.convert.support.AbstractConditionalEnumConverter
///
/// Spring 语义：携带枚举类型信息并提供 `matches` 条件判定；Rust 中以
/// `FromStr + Copy` 表达可匹配的枚举类型族。
pub trait AbstractConditionalEnumConverter: Send + Sync {
    /// 把字符串解析为枚举值。
    ///
    /// 对标 Spring 子类 `convert` 的枚举解析路径。
    fn convert_enum<E>(&self, value: &str) -> Result<E, ConversionError>
    where
        E: std::str::FromStr,
        E::Err: std::fmt::Display,
    {
        value.parse::<E>().map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: std::any::type_name::<E>(),
            reason: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleEnumConverter;

    impl AbstractConditionalEnumConverter for SampleEnumConverter {}

    #[derive(Debug, PartialEq)]
    enum Sample {
        A,
        B,
    }

    impl std::str::FromStr for Sample {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "a" => Ok(Sample::A),
                "b" => Ok(Sample::B),
                _ => Err(format!("unknown: {s}")),
            }
        }
    }

    #[test]
    fn base_conversion_works() {
        // A 类（合同对齐）：对标 Spring 抽象基类的枚举解析
        let converter = SampleEnumConverter;
        assert_eq!(converter.convert_enum::<Sample>("a").unwrap(), Sample::A);
    }

    #[test]
    fn unknown_value_returns_error() {
        // C 类（错误路径）
        let converter = SampleEnumConverter;
        assert!(converter.convert_enum::<Sample>("z").is_err());
    }
}
