//! 转换器未找到异常。
//!
//! 对标 Spring `org.springframework.core.convert.ConverterNotFoundException`。

use std::any::TypeId;
use std::fmt;

/// 转换器未找到异常。
///
/// 对应 Java: org.springframework.core.convert.ConverterNotFoundException
///
/// Spring 语义：`ConversionService.convert` 找不到匹配转换器时抛出，携带
/// 源/目标类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConverterNotFoundException {
    /// 源类型 ID。
    pub source_type: TypeId,
    /// 目标类型 ID。
    pub target_type: TypeId,
}

impl ConverterNotFoundException {
    /// 创建未找到异常。
    #[must_use]
    pub fn new(source_type: TypeId, target_type: TypeId) -> Self {
        Self {
            source_type,
            target_type,
        }
    }
}

impl fmt::Display for ConverterNotFoundException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "找不到从 {:?} 到 {:?} 的转换器",
            self.source_type, self.target_type
        )
    }
}

impl std::error::Error for ConverterNotFoundException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_source_and_target_types() {
        // A 类（合同对齐）：对标 Spring 异常携带类型信息
        let err = ConverterNotFoundException::new(TypeId::of::<String>(), TypeId::of::<bool>());
        assert_eq!(err.source_type, TypeId::of::<String>());
        assert_eq!(err.target_type, TypeId::of::<bool>());
    }

    #[test]
    fn displays_type_ids() {
        // B 类（边界行为）：消息可诊断
        let err = ConverterNotFoundException::new(TypeId::of::<String>(), TypeId::of::<bool>());
        assert!(err.to_string().contains("转换器"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ConverterNotFoundException>();
    }
}
