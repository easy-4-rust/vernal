//! 字符串转换服务。
//!
//! 对标 Spring `DefaultConversionService` 简化版。

use super::convertible::Convertible;
use super::conversion_error::ConversionError;

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

    /// 获取全局共享的 ConversionService 实例。
    ///
    /// 对标 Spring `DefaultConversionService.getSharedInstance()`。
    #[must_use]
    pub fn get_shared_instance() -> &'static Self {
        static INSTANCE: ConversionService = ConversionService;
        &INSTANCE
    }
}
