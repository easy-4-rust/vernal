//! 类型转换模块（精简版）。
//!
//! 仅包含配置属性绑定必需的 5 个核心转换器。
//! 对标 Spring 的 `ConversionService`，但只实现最小集合。
//!
//! # 设计原则
//!
//! - 仅实现框架**必须**的转换（配置属性绑定）
//! - 不实现通用转换（UUID/Date/集合等留给 hutool-rust）
//! - 纯 Rust 实现，不依赖外部库

mod converter;
mod string_converter;
mod number_converter;
mod boolean_converter;
mod enum_converter;
mod option_converter;

pub use converter::Converter;
pub use string_converter::StringConverter;
pub use number_converter::NumberConverter;
pub use boolean_converter::BooleanConverter;
pub use enum_converter::convert_enum;
pub use option_converter::OptionConverter;

/// 简化版类型转换服务。
///
/// 仅支持配置属性绑定必需的 5 种转换。
/// 对标 Spring 的 `DefaultConversionService`。
pub struct ConversionService;

impl ConversionService {
    /// 将字符串值转换为目标类型。
    ///
    /// # 支持的转换
    ///
    /// - `String` → `String`（直接返回）
    /// - `String` → `i32/i64/u32/u64/f32/f64/isize/usize`（数字解析）
    /// - `String` → `bool`（"true"/"false"/"1"/"0"）
    /// - `String` → 任意实现了 `FromStr` 的枚举
    /// - `Option<T>` → `Option<T>`（递归转换）
    pub fn convert<T: Convertible>(value: &str) -> Result<T, ConversionError> {
        T::from_str_value(value)
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
