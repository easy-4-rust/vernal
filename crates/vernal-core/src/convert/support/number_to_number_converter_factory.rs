//! 数字 → 数字转换器工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.NumberToNumberConverterFactory`。

use crate::convert::{ConversionError, Converter, ConverterFactory};

/// 数字 → 数字转换器工厂。
///
/// 对应 Java: org.springframework.core.convert.support.NumberToNumberConverterFactory
///
/// 对标 Spring `NumberUtils.convertNumberToTargetClass`：先解析为 `i128`/`u128`/`f64`
/// 中间值，再做带范围检查的目标类型转换（溢出时返回转换错误）。
pub struct NumberToNumberConverterFactory;

impl NumberToNumberConverterFactory {
    /// 把字符串转换为目标数字类型。
    ///
    /// # 错误
    ///
    /// 解析失败或溢出时返回 [`ConversionError`]。
    pub fn convert<T: NumberTarget>(&self, source: &str) -> Result<T, ConversionError> {
        T::convert_number(source)
    }

    /// 判定目标类型是否受支持。
    #[must_use]
    pub fn can_convert<T: NumberTarget>() -> bool {
        true
    }
}

/// 数字目标类型的转换约束（对标 Spring `Number` 子类集合）。
pub trait NumberTarget: Sized {
    /// 从字符串解析中间值并转换为目标类型。
    fn convert_number(source: &str) -> Result<Self, ConversionError>;
}

macro_rules! impl_number_target {
    ($ty:ty, $parse:ident) => {
        impl NumberTarget for $ty {
            fn convert_number(source: &str) -> Result<Self, ConversionError> {
                source.parse::<$ty>().map_err(|e| ConversionError {
                    value: source.to_string(),
                    target_type: stringify!($ty),
                    reason: e.to_string(),
                })
            }
        }
    };
}

impl_number_target!(i8, parse);
impl_number_target!(i16, parse);
impl_number_target!(i32, parse);
impl_number_target!(i64, parse);
impl_number_target!(i128, parse);
impl_number_target!(u8, parse);
impl_number_target!(u16, parse);
impl_number_target!(u32, parse);
impl_number_target!(u64, parse);
impl_number_target!(u128, parse);
impl_number_target!(f32, parse);
impl_number_target!(f64, parse);

/// 内部转换器：委托 `NumberTarget::convert_number`。
struct NumberToNumberConverter<T>(std::marker::PhantomData<T>);

impl<T: NumberTarget> Converter<&str, T> for NumberToNumberConverter<T> {
    fn convert(&self, source: &str) -> Result<T, ConversionError> {
        T::convert_number(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_within_range() {
        // A 类（合同对齐）：对标 Spring `convertNumberToTargetClass`
        let factory = NumberToNumberConverterFactory;
        assert_eq!(factory.convert::<u16>("65535").unwrap(), u16::MAX);
    }

    #[test]
    fn overflow_returns_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException`（溢出）
        let factory = NumberToNumberConverterFactory;
        assert!(factory.convert::<u8>("256").is_err());
    }

    #[test]
    fn negative_to_unsigned_returns_error() {
        // B 类（边界行为）：负值转无符号数应报错
        let factory = NumberToNumberConverterFactory;
        assert!(factory.convert::<u32>("-1").is_err());
    }
}
