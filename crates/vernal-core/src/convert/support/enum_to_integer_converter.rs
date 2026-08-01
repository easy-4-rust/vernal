//! 枚举 → 整数转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.EnumToIntegerConverter`。

use crate::convert::{ConversionError, Converter};

/// 枚举 → 整数转换器。
///
/// 对应 Java: org.springframework.core.convert.support.EnumToIntegerConverter
///
/// Spring 语义：`Enum.ordinal()`——变体声明序号。
pub struct EnumToIntegerConverter;

/// 提供变体序号的枚举抽象（对标 Java `Enum.ordinal()`）。
pub trait Enumerated {
    /// 变体声明序号（从 0 开始）。
    fn ordinal(&self) -> u32;
}

impl<T: Enumerated> Converter<&T, u32> for EnumToIntegerConverter {
    fn convert(&self, source: &T) -> Result<u32, ConversionError> {
        Ok(source.ordinal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum Status {
        Pending,
        Running,
        Done,
    }

    impl Enumerated for Status {
        fn ordinal(&self) -> u32 {
            match self {
                Status::Pending => 0,
                Status::Running => 1,
                Status::Done => 2,
            }
        }
    }

    #[test]
    fn converts_ordinal() {
        // A 类（合同对齐）：对标 Spring `Enum.ordinal()`
        let converter = EnumToIntegerConverter;
        assert_eq!(converter.convert(&Status::Running).unwrap(), 1);
    }

    #[test]
    fn first_variant_is_zero() {
        // B 类（边界行为）：首个变体序号为 0
        let converter = EnumToIntegerConverter;
        assert_eq!(converter.convert(&Status::Pending).unwrap(), 0);
    }
}
