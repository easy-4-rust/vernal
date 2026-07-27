//! 通用转换器 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.GenericConverter`。
//!
//! # 设计来源
//!
//! Spring 的 `GenericConverter` 是最灵活但也最复杂的转换 SPI:
//! - 一个 `GenericConverter` 可以支持多对 source/target 类型(`getConvertibleTypes()`)
//! - 转换过程中可访问 `TypeDescriptor` 字段元数据(注解、泛型信息)
//!
//! vernal-core 简化为基于 `TypeId` 的多对多转换器抽象。
//!
//! # 与 `Converter<S, T>` 的关系
//!
//! | 特性 | `Converter<S, T>` | `GenericConverter` |
//! |---|---|---|
//! | 类型对数 | 单一(S→T) | 多对(Set<ConvertiblePair>) |
//! | 类型擦除 | 静态(编译期) | 运行时(TypeId 匹配) |
//! | 字段元数据 | 不可访问 | 可访问(对标 TypeDescriptor) |
//! | 推荐场景 | 简单单向转换 | 复杂多对多转换 |

use std::any::TypeId;
use std::collections::HashSet;

use super::{ConditionalConverter, ConversionError};

/// 可转换类型对。
///
/// 对标 Spring `GenericConverter.ConvertiblePair`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConvertiblePair {
    /// 源类型 ID
    pub source: TypeId,
    /// 目标类型 ID
    pub target: TypeId,
}

impl ConvertiblePair {
    /// 创建新的类型对。
    #[must_use]
    pub fn new<S: 'static, T: 'static>() -> Self {
        Self {
            source: TypeId::of::<S>(),
            target: TypeId::of::<T>(),
        }
    }

    /// 从显式 `TypeId` 创建。
    #[must_use]
    pub fn from_type_ids(source: TypeId, target: TypeId) -> Self {
        Self { source, target }
    }
}

/// 通用转换器 trait。
///
/// 对标 Spring `GenericConverter`。
///
/// 一个实现可以支持多个类型对(`convertible_types`),并根据运行时类型匹配决定是否执行。
pub trait GenericConverter: ConditionalConverter {
    /// 返回此转换器支持的所有类型对。
    ///
    /// 对标 Spring `Set<ConvertiblePair> getConvertibleTypes()`。
    /// 返回空集合表示支持所有类型对(仅当 `matches` 返回 true 时才执行)。
    fn convertible_types(&self) -> HashSet<ConvertiblePair>;

    /// 执行转换。
    ///
    /// 对标 Spring `Object convert(Object source, TypeDescriptor sourceType, TypeDescriptor targetType)`。
    ///
    /// vernal-core 简化为字符串→字符串(因为 vernal-core 的 `Convertible` trait 基于 `from_str_value`)。
    fn convert(&self, source: &str, target_type: TypeId) -> Result<String, ConversionError>;
}

/// 基于闭包的简单 `GenericConverter`。
///
/// 适用于需要运行时多对多转换但逻辑简单的场景。
pub struct ClosureGenericConverter {
    pairs: HashSet<ConvertiblePair>,
    closure: Box<dyn Fn(&str, TypeId) -> Result<String, ConversionError> + Send + Sync>,
}

impl ClosureGenericConverter {
    /// 创建新的闭包转换器。
    #[must_use]
    pub fn new<F>(pairs: HashSet<ConvertiblePair>, closure: F) -> Self
    where
        F: Fn(&str, TypeId) -> Result<String, ConversionError> + Send + Sync + 'static,
    {
        Self {
            pairs,
            closure: Box::new(closure),
        }
    }
}

impl ConditionalConverter for ClosureGenericConverter {
    fn matches(&self, source_type: TypeId, target_type: TypeId) -> bool {
        if self.pairs.is_empty() {
            return true;
        }
        self.pairs
            .iter()
            .any(|p| p.source == source_type && p.target == target_type)
    }
}

impl GenericConverter for ClosureGenericConverter {
    fn convertible_types(&self) -> HashSet<ConvertiblePair> {
        self.pairs.clone()
    }

    fn convert(&self, source: &str, target_type: TypeId) -> Result<String, ConversionError> {
        (self.closure)(source, target_type)
    }
}

impl std::fmt::Debug for ClosureGenericConverter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureGenericConverter")
            .field("pairs_count", &self.pairs.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convertible_pair_equality() {
        let p1 = ConvertiblePair::new::<String, i64>();
        let p2 = ConvertiblePair::new::<String, i64>();
        let p3 = ConvertiblePair::new::<String, bool>();
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn convertible_pair_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ConvertiblePair::new::<String, i64>());
        set.insert(ConvertiblePair::new::<String, i64>());
        set.insert(ConvertiblePair::new::<String, bool>());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn closure_converter_matches_registered_pair() {
        let mut pairs = HashSet::new();
        pairs.insert(ConvertiblePair::new::<String, i64>());
        let converter = ClosureGenericConverter::new(pairs, |s, _| Ok(s.to_string()));

        assert!(converter.matches(TypeId::of::<String>(), TypeId::of::<i64>()));
        assert!(!converter.matches(TypeId::of::<String>(), TypeId::of::<bool>()));
    }

    #[test]
    fn closure_converter_with_empty_pairs_matches_all() {
        let converter = ClosureGenericConverter::new(HashSet::new(), |s, _| Ok(s.to_string()));
        assert!(converter.matches(TypeId::of::<String>(), TypeId::of::<i64>()));
        assert!(converter.matches(TypeId::of::<bool>(), TypeId::of::<f64>()));
    }

    #[test]
    fn closure_converter_executes_closure() {
        let pairs = HashSet::from([ConvertiblePair::new::<String, i64>()]);
        let converter = ClosureGenericConverter::new(pairs, |s, _| {
            Ok(s.parse::<i64>().unwrap_or(0).to_string())
        });
        let result = converter.convert("42", TypeId::of::<i64>()).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn generic_converter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ClosureGenericConverter>();
    }
}
