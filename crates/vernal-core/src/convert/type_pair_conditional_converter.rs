//! 类型对条件转换器。
//!
//! 对标 Spring `ConverterAdapter` 与 `ConditionalConverter` 桥接。

use super::conditional_converter::ConditionalConverter;
use super::convertible_pair::ConvertiblePair;

/// 类型对条件转换器。
///
/// 对应 Java: `ConditionalConverter` + `GenericConverter` 的实现桥接
#[derive(Debug, Clone, Copy)]
pub struct TypePairConditionalConverter {
    source_type_id: std::any::TypeId,
    target_type_id: std::any::TypeId,
}

impl TypePairConditionalConverter {
    /// 创建新的类型对条件转换器。
    #[must_use]
    pub fn new<S: 'static, T: 'static>() -> Self {
        Self {
            source_type_id: std::any::TypeId::of::<S>(),
            target_type_id: std::any::TypeId::of::<T>(),
        }
    }

    /// 获取源类型 `TypeId`。
    #[must_use]
    pub fn source_type(&self) -> std::any::TypeId {
        self.source_type_id
    }

    /// 获取目标类型 `TypeId`。
    #[must_use]
    pub fn target_type(&self) -> std::any::TypeId {
        self.target_type_id
    }
}

impl ConditionalConverter for TypePairConditionalConverter {
    fn matches(&self, pair: &ConvertiblePair) -> bool {
        self.source_type_id == pair.source_type_id()
            && self.target_type_id == pair.target_type_id()
    }
}
