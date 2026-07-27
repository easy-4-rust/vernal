//! 条件转换器 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConditionalConverter`。
//!
//! # 设计来源
//!
//! Spring 的 `ConditionalConverter` 允许 `Converter` / `GenericConverter` /
//! `ConverterFactory` 在执行前根据 source/target 的 `TypeDescriptor` 决定是否执行。
//!
//! 典型用例:
//! - String→Date 仅在目标字段有 `@DateTimeFormat` 注解时执行
//! - String→Account 仅在 Account 类有 `static findAccount(String)` 方法时执行
//!
//! # Rust 适配
//!
//! Rust 没有运行时注解,但可以通过 trait bound(`where T: SomeMarker`)或
//! 类型谓词函数(`fn matches<T>() -> bool`)实现等价语义。
//! vernal-core 提供 `ConditionalConverter` trait 作为运行时可配置的条件检查。

use std::any::TypeId;

/// 条件转换器 trait。
///
/// 对标 Spring `ConditionalConverter`。
///
/// 实现此 trait 后,转换器会在 [`Self::matches`] 返回 `true` 时才执行。
pub trait ConditionalConverter: Send + Sync {
    /// 判断指定类型对是否应执行此转换器。
    ///
    /// 对标 Spring `boolean matches(TypeDescriptor sourceType, TypeDescriptor targetType)`。
    ///
    /// # 参数
    ///
    /// - `source_type`:源类型的 `TypeId`
    /// - `target_type`:目标类型的 `TypeId`
    fn matches(&self, source_type: TypeId, target_type: TypeId) -> bool;
}

/// 基于类型对的简单条件转换器。
///
/// 只有当 source/target 类型与构造时指定的类型对完全匹配时返回 `true`。
#[derive(Debug, Clone)]
pub struct TypePairConditionalConverter {
    source: TypeId,
    target: TypeId,
}

impl TypePairConditionalConverter {
    /// 创建条件转换器,指定唯一的匹配类型对。
    #[must_use]
    pub fn new<S: 'static, T: 'static>() -> Self {
        Self {
            source: TypeId::of::<S>(),
            target: TypeId::of::<T>(),
        }
    }

    /// 获取源类型 ID。
    #[must_use]
    pub fn source_type(&self) -> TypeId {
        self.source
    }

    /// 获取目标类型 ID。
    #[must_use]
    pub fn target_type(&self) -> TypeId {
        self.target
    }
}

impl ConditionalConverter for TypePairConditionalConverter {
    fn matches(&self, source_type: TypeId, target_type: TypeId) -> bool {
        source_type == self.source && target_type == self.target
    }
}

/// 总是匹配的条件转换器(无条件)。
#[derive(Debug, Clone, Copy)]
pub struct AlwaysMatchConverter;

impl ConditionalConverter for AlwaysMatchConverter {
    fn matches(&self, _source_type: TypeId, _target_type: TypeId) -> bool {
        true
    }
}

/// 从不匹配的条件转换器(禁用)。
#[derive(Debug, Clone, Copy)]
pub struct NeverMatchConverter;

impl ConditionalConverter for NeverMatchConverter {
    fn matches(&self, _source_type: TypeId, _target_type: TypeId) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_pair_converter_matches_exact_pair() {
        let c = TypePairConditionalConverter::new::<String, i64>();
        assert!(c.matches(TypeId::of::<String>(), TypeId::of::<i64>()));
    }

    #[test]
    fn type_pair_converter_rejects_wrong_pair() {
        let c = TypePairConditionalConverter::new::<String, i64>();
        assert!(!c.matches(TypeId::of::<String>(), TypeId::of::<bool>()));
        assert!(!c.matches(TypeId::of::<i32>(), TypeId::of::<i64>()));
    }

    #[test]
    fn always_match_converter_returns_true() {
        let c = AlwaysMatchConverter;
        assert!(c.matches(TypeId::of::<String>(), TypeId::of::<i64>()));
        assert!(c.matches(TypeId::of::<bool>(), TypeId::of::<f64>()));
    }

    #[test]
    fn never_match_converter_returns_false() {
        let c = NeverMatchConverter;
        assert!(!c.matches(TypeId::of::<String>(), TypeId::of::<i64>()));
    }

    #[test]
    fn source_target_accessors_work() {
        let c = TypePairConditionalConverter::new::<String, i64>();
        assert_eq!(c.source_type(), TypeId::of::<String>());
        assert_eq!(c.target_type(), TypeId::of::<i64>());
    }

    #[test]
    fn conditional_converter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TypePairConditionalConverter>();
        assert_send_sync::<AlwaysMatchConverter>();
        assert_send_sync::<NeverMatchConverter>();
    }
}
