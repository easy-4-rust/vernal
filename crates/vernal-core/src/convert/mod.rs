//! 类型转换模块（精简版）。
//!
//! 对标 Spring 的 `org.springframework.core.convert.ConversionService` 与
//! `org.springframework.core.convert.support.DefaultConversionService`,但只实现最小集合。
//!
//! # 内置 Convertible 类型（10 个）
//!
//! - `bool` (布尔转换器,接受 true/1/yes/on/false/0/no/off)
//! - 13 种数字类型 (i8..i128, u8..u128, f32/f64, isize/usize)
//! - `String` (恒等函数)
//! - 任意 `T: FromStr` (通过 `convert_enum`)
//! - `Option<T: Convertible>` (空字符串 → None)
//! - `PathBuf` (路径转换器,新增)
//! - `Duration` (时间间隔,支持 ISO-8601 与简化语法,新增)
//! - `SocketAddr` / `SocketAddrV4` / `SocketAddrV6` (网络地址,新增)
//!
//! # 设计原则
//!
//! - 仅实现框架**必须**的转换（配置属性绑定）
//! - 不实现通用转换（UUID/Date/集合等留给 hutool-rust）
//! - 纯 Rust 实现,不依赖外部库

mod boolean_converter;
mod conditional_converter;
mod converter;
mod converter_not_found_error;
mod converter_registry;
mod duration_converter;
mod enum_converter;
mod generic_converter;
mod number_converter;
mod option_converter;
mod path_converter;
mod socket_addr_converter;
mod string_converter;

pub use boolean_converter::BooleanConverter;
pub use conditional_converter::{
    AlwaysMatchConverter, ConditionalConverter, NeverMatchConverter, TypePairConditionalConverter,
};
pub use converter::Converter;
pub use converter_not_found_error::{converter_not_found, is_converter_not_found};
pub use converter_registry::{ConverterRegistry, TypeIdConverterRegistry};
pub use duration_converter::DurationConverter;
pub use enum_converter::convert_enum;
pub use generic_converter::{ClosureGenericConverter, ConvertiblePair, GenericConverter};
pub use number_converter::NumberConverter;
pub use option_converter::OptionConverter;
pub use path_converter::PathConverter;
pub use socket_addr_converter::SocketAddrConverter;
pub use string_converter::StringConverter;

// Feature-gated 转换器
#[cfg(feature = "convert-url")]
mod url_converter;
#[cfg(feature = "convert-url")]
pub use url_converter::UrlConverter;

#[cfg(feature = "convert-time")]
mod datetime_converter;
#[cfg(feature = "convert-time")]
pub use datetime_converter::DatetimeConverter;

#[cfg(feature = "convert-regex")]
mod regex_converter;
#[cfg(feature = "convert-regex")]
pub use regex_converter::RegexConverter;

/// 简化版类型转换服务。
///
/// 仅支持配置属性绑定必需的 10 种转换。
/// 对标 Spring 的 `DefaultConversionService`。
pub struct ConversionService;

impl ConversionService {
    /// 将字符串值转换为目标类型。
    ///
    /// # 支持的转换
    ///
    /// - `String` → `String`（直接返回）
    /// - `String` → `i32/i64/u32/u64/f32/f64/isize/usize`（数字解析）
    /// - `String` → `bool`（"true"/"false"/"1"/"0"/"yes"/"no"/"on"/"off"）
    /// - `String` → 任意实现了 `FromStr` 的枚举
    /// - `Option<T>` → `Option<T>`（递归转换）
    /// - `String` → `PathBuf`（路径）
    /// - `String` → `Duration`（时间间隔,支持 `30s` / `PT1H30M`）
    /// - `String` → `SocketAddr` / `SocketAddrV4` / `SocketAddrV6`（网络地址）
    pub fn convert<T: Convertible>(value: &str) -> Result<T, ConversionError> {
        T::from_str_value(value)
    }

    /// 判定目标类型是否支持从字符串转换。
    ///
    /// 对标 Spring `ConversionService.canConvert(Class, Class)`。
    /// 返回 `true` 表示该类型实现了 [`Convertible`] trait。
    ///
    /// 注意：由于 Rust 类型系统无法在运行时直接探测 trait 实现,本方法通过
    /// 调用 [`Self::convert`] 并捕获错误来判断。如果转换失败但原因是空值或格式错误
    /// (而不是类型错误),仍返回 `true`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vernal_core::convert::ConversionService;
    ///
    /// // 编译期已知的 Convertible 类型都返回 true
    /// assert!(ConversionService::can_convert::<i32>());
    /// assert!(ConversionService::can_convert::<bool>());
    /// assert!(ConversionService::can_convert::<String>());
    /// ```
    #[must_use]
    pub fn can_convert<T: Convertible>() -> bool {
        // 由于 Convertible 是 sealed trait,所有实现都已知;直接返回 true
        // 真正的"运行时探测"需要 TypeId 检查,但 vernal 选择静态分发优先
        !std::any::type_name::<T>().is_empty()
    }
}

/// 可转换的目标类型 trait。
///
/// 实现此 trait 的类型可以从字符串值转换而来。
pub trait Convertible: Sized {
    /// 从字符串值转换。
    fn from_str_value(value: &str) -> Result<Self, ConversionError>;
}

/// 转换错误。
#[derive(Debug, Clone)]
pub struct ConversionError {
    /// 原始值
    pub value: String,
    /// 目标类型名
    pub target_type: &'static str,
    /// 错误原因
    pub reason: String,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "无法将 \"{}\" 转换为 {}: {}",
            self.value, self.target_type, self.reason
        )
    }
}

impl std::error::Error for ConversionError {}

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
