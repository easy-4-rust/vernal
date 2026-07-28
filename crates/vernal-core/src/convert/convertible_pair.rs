//! 类型对。
//!
//! 对标 Spring `GenericConverter.ConvertiblePair`。

/// 类型对，记录转换的源类型与目标类型。
///
/// 对应 Java: `GenericConverter.ConvertiblePair`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvertiblePair {
    /// 源类型 `TypeId`
    pub source_type_id: std::any::TypeId,
    /// 目标类型 `TypeId`
    pub target_type_id: std::any::TypeId,
}

impl ConvertiblePair {
    /// 创建新的类型对。
    #[must_use]
    pub fn new<S: 'static, T: 'static>() -> Self {
        Self {
            source_type_id: std::any::TypeId::of::<S>(),
            target_type_id: std::any::TypeId::of::<T>(),
        }
    }

    /// 从 `TypeId` 创建。
    #[must_use]
    pub fn from_type_ids(source: std::any::TypeId, target: std::any::TypeId) -> Self {
        Self {
            source_type_id: source,
            target_type_id: target,
        }
    }

    /// 获取源类型 `TypeId`。
    #[must_use]
    pub fn source_type_id(&self) -> std::any::TypeId {
        self.source_type_id
    }

    /// 获取目标类型 `TypeId`。
    #[must_use]
    pub fn target_type_id(&self) -> std::any::TypeId {
        self.target_type_id
    }
}
