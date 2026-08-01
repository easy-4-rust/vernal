//! 字符串转换模块。
//!
//! 对标 Spring `org.springframework.core.convert` 包。

mod boolean_converter;
mod conversion_error;
mod conversion_service;
mod convertible;
mod converter;
mod converter_not_found_error;
mod duration_converter;
mod enum_converter;
mod number_converter;
mod option_converter;
mod path_converter;
mod socket_addr_converter;
mod string_converter;

#[cfg(feature = "convert-bytes")]
mod bytes_converter;
#[cfg(feature = "convert-time")]
mod datetime_converter;
#[cfg(feature = "convert-regex")]
mod regex_converter;
#[cfg(feature = "convert-url")]
mod url_converter;
#[cfg(feature = "convert-uuid")]
mod uuid_converter;

pub use boolean_converter::BooleanConverter;
pub use conversion_error::ConversionError;
pub use conversion_service::ConversionService;
pub use convertible::Convertible;
pub use converter::Converter;
pub use converter_not_found_error::{converter_not_found, is_converter_not_found};
pub use converter::{
    AlwaysMatchConverter, ClosureGenericConverter, ConditionalConverter, ConvertiblePair,
    ConverterRegistry, GenericConverter, NeverMatchConverter, TypeIdConverterRegistry,
    TypePairConditionalConverter,
};
pub use duration_converter::DurationConverter;
pub use enum_converter::convert_enum;
pub use number_converter::NumberConverter;
pub use option_converter::OptionConverter;
pub use path_converter::PathConverter;
pub use socket_addr_converter::SocketAddrConverter;
pub use string_converter::StringConverter;

#[cfg(feature = "convert-time")]
pub use datetime_converter::DatetimeConverter;

#[cfg(feature = "convert-regex")]
pub use regex_converter::RegexConverter;

#[cfg(feature = "convert-url")]
pub use url_converter::UrlConverter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_returns_true_for_known_types() {
        assert!(ConversionService::can_convert::<bool>());
        assert!(ConversionService::can_convert::<i32>());
        assert!(ConversionService::can_convert::<u64>());
        assert!(ConversionService::can_convert::<f64>());
        assert!(ConversionService::can_convert::<String>());
        assert!(ConversionService::can_convert::<std::path::PathBuf>());
        assert!(ConversionService::can_convert::<std::time::Duration>());
        assert!(ConversionService::can_convert::<std::net::SocketAddr>());
        assert!(ConversionService::can_convert::<Option<i32>>());
    }

    #[test]
    fn conversion_error_display_chinese() {
        let err = ConversionError {
            value: "abc".to_string(),
            target_type: "i32",
            reason: "数字格式无效".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("abc"));
        assert!(s.contains("i32"));
        assert!(s.contains("数字格式无效"));
    }

    #[test]
    fn conversion_error_is_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ConversionError>();
    }

    #[test]
    fn convertible_trait_can_be_implemented_by_user() {
        // 验证用户可以为自定义类型实现 Convertible
        #[derive(Debug, PartialEq)]
        enum Mode {
            Production,
            Development,
        }

        impl std::str::FromStr for Mode {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "prod" => Ok(Mode::Production),
                    "dev" => Ok(Mode::Development),
                    _ => Err(format!("unknown mode: {s}")),
                }
            }
        }

        impl Convertible for Mode {
            fn from_str_value(value: &str) -> Result<Self, ConversionError> {
                value.parse().map_err(|e| ConversionError {
                    value: value.to_string(),
                    target_type: "Mode",
                    reason: e,
                })
            }
        }

        let mode: Mode = ConversionService::convert("prod").unwrap();
        assert_eq!(mode, Mode::Production);

        let mode: Mode = ConversionService::convert("dev").unwrap();
        assert_eq!(mode, Mode::Development);

        let err = ConversionService::convert::<Mode>("staging").unwrap_err();
        assert_eq!(err.target_type, "Mode");
    }
}
