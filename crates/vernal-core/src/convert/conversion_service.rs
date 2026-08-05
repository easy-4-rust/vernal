//! 字符串转换服务。
//!
//! 对标 Spring `DefaultConversionService` 简化版。

use super::conversion_error::ConversionError;
use super::convertible::Convertible;

/// 字符串转换服务。
///
/// 对应 Java: org.springframework.core.convert.support.DefaultConversionService
///
/// 由于 Rust 类型系统采用静态分发，`ConversionService` 的核心能力通过
/// [`Convertible`] trait 的关联函数提供（[`ConversionService::convert`] 等）。
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
    #[must_use]
    pub fn can_convert<T: Convertible>() -> bool {
        !std::any::type_name::<T>().is_empty()
    }

    /// 获取全局共享的 `ConversionService` 实例。
    ///
    /// 对标 Spring `DefaultConversionService.getSharedInstance()`。
    #[must_use]
    pub fn get_shared_instance() -> &'static Self {
        static INSTANCE: ConversionService = ConversionService;
        &INSTANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_string_to_string() {
        let result: String = ConversionService::convert("hello").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn convert_string_to_i32() {
        let result: i32 = ConversionService::convert("42").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn convert_string_to_i64() {
        let result: i64 = ConversionService::convert("9999999999").unwrap();
        assert_eq!(result, 9_999_999_999);
    }

    #[test]
    fn convert_string_to_f64() {
        let result: f64 = ConversionService::convert("3.25").unwrap();
        assert!((result - 3.25).abs() < 1e-10);
    }

    #[test]
    fn convert_string_to_bool_true() {
        let result: bool = ConversionService::convert("true").unwrap();
        assert!(result);
    }

    #[test]
    fn convert_string_to_bool_false() {
        let result: bool = ConversionService::convert("false").unwrap();
        assert!(!result);
    }

    #[test]
    fn convert_invalid_string_returns_error() {
        let result: Result<i32, _> = ConversionService::convert("not-a-number");
        assert!(result.is_err());
    }

    #[test]
    fn can_convert_returns_true_for_supported_types() {
        assert!(ConversionService::can_convert::<String>());
        assert!(ConversionService::can_convert::<i32>());
        assert!(ConversionService::can_convert::<i64>());
        assert!(ConversionService::can_convert::<f64>());
        assert!(ConversionService::can_convert::<bool>());
    }

    #[test]
    fn get_shared_instance_returns_same_reference() {
        let a = ConversionService::get_shared_instance();
        let b = ConversionService::get_shared_instance();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn convert_string_to_option_some() {
        let result: Option<String> = ConversionService::convert("hello").unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn convert_empty_string_to_option_returns_none() {
        let result: Option<i32> = ConversionService::convert("").unwrap();
        assert_eq!(result, None);
    }
}
