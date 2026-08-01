//! 类型对。
//!
//! 对标 Spring `GenericConverter.ConvertiblePair`。

/// 类型对，记录转换的源类型与目标类型。
///
/// 对应 Java: `GenericConverter.ConvertiblePair`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 对标 Spring `ConvertiblePair.getSourceType()` / `getTargetType()`
    #[test]
    fn source_and_target_type_id_accessors_return_constructed_ids() {
        let pair = ConvertiblePair::new::<String, i64>();
        let expected_source = std::any::TypeId::of::<String>();
        let expected_target = std::any::TypeId::of::<i64>();
        assert_eq!(pair.source_type_id(), expected_source);
        assert_eq!(pair.target_type_id(), expected_target);
    }

    /// 对标 Spring `ConvertiblePair.equals`: 相同类型对相等
    #[test]
    fn equality_holds_for_same_type_args() {
        let a = ConvertiblePair::new::<u32, String>();
        let b = ConvertiblePair::new::<u32, String>();
        assert_eq!(a, b);
    }

    /// 源类型不同则不相等
    #[test]
    fn inequality_when_source_differs() {
        let a = ConvertiblePair::new::<u32, String>();
        let b = ConvertiblePair::new::<i32, String>();
        assert_ne!(a, b);
    }

    /// 目标类型不同则不相等
    #[test]
    fn inequality_when_target_differs() {
        let a = ConvertiblePair::new::<u32, String>();
        let b = ConvertiblePair::new::<u32, Vec<u8>>();
        assert_ne!(a, b);
    }

    /// Hash 一致性: 相等的 pair 必须有相同的 hash（HashMap/HashSet 依赖）
    #[test]
    fn hash_is_consistent_with_equality() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let a = ConvertiblePair::new::<String, i64>();
        let b = ConvertiblePair::new::<String, i64>();
        let mut h1 = DefaultHasher::new();
        a.hash(&mut h1);
        let mut h2 = DefaultHasher::new();
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    /// Copy 语义验证
    #[test]
    fn is_copy_semantics() {
        let pair = ConvertiblePair::new::<String, i64>();
        let copy1 = pair;
        let copy2 = pair; // 仍然可用, 因为 Copy
        assert_eq!(copy1, copy2);
        assert_eq!(copy1.source_type_id(), std::any::TypeId::of::<String>());
    }

    /// `from_type_ids` 显式构造
    #[test]
    fn from_type_ids_explicit_construction() {
        let s = std::any::TypeId::of::<String>();
        let t = std::any::TypeId::of::<bool>();
        let pair = ConvertiblePair::from_type_ids(s, t);
        assert_eq!(pair.source_type_id(), s);
        assert_eq!(pair.target_type_id(), t);
        // 跨构造路径等价
        assert_eq!(pair, ConvertiblePair::new::<String, bool>());
    }
}
