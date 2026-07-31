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

use super::convertible_pair::ConvertiblePair;
use super::{ConditionalConverter, ConversionError};



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
    fn matches(&self, pair: &ConvertiblePair) -> bool {
        if self.pairs.is_empty() {
            return true;
        }
        self.pairs.iter().any(|p| {
            p.source_type_id() == pair.source_type_id()
                && p.target_type_id() == pair.target_type_id()
        })
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

        assert!(converter.matches(&ConvertiblePair::new::<String, i64>()));
        assert!(!converter.matches(&ConvertiblePair::new::<String, bool>()));
    }

    #[test]
    fn closure_converter_with_empty_pairs_matches_all() {
        let converter = ClosureGenericConverter::new(HashSet::new(), |s, _| Ok(s.to_string()));
        assert!(converter.matches(&ConvertiblePair::new::<String, i64>()));
        assert!(converter.matches(&ConvertiblePair::new::<bool, f64>()));
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

    #[test]
    fn convertible_types_returns_registered_pairs() {
        // 对标 Spring `getConvertibleTypes()` 返回支持的类型对集合
        let mut pairs = HashSet::new();
        pairs.insert(ConvertiblePair::new::<String, i64>());
        pairs.insert(ConvertiblePair::new::<String, bool>());
        let converter = ClosureGenericConverter::new(pairs, |s, _| Ok(s.to_string()));
        let returned = converter.convertible_types();
        assert_eq!(returned.len(), 2);
        assert!(returned.contains(&ConvertiblePair::new::<String, i64>()));
        assert!(returned.contains(&ConvertiblePair::new::<String, bool>()));
    }

    #[test]
    fn convertible_types_is_a_clone_independent_from_internal() {
        // 修改返回的集合不应影响 converter 内部状态
        let mut pairs = HashSet::new();
        pairs.insert(ConvertiblePair::new::<String, i64>());
        let converter = ClosureGenericConverter::new(pairs, |s, _| Ok(s.to_string()));
        let mut returned = converter.convertible_types();
        // 再调用一次, 集合大小不变
        let second = converter.convertible_types();
        assert_eq!(returned.len(), 1);
        assert_eq!(second.len(), 1);
        // 拿到的 HashSet 是独立副本
        returned.insert(ConvertiblePair::new::<Vec<u8>, i64>());
        assert_eq!(converter.convertible_types().len(), 1);
    }

    #[test]
    fn debug_format_includes_pairs_count() {
        // 对标 Spring `GenericConverter.toString()` 风格
        let mut pairs = HashSet::new();
        pairs.insert(ConvertiblePair::new::<String, i64>());
        pairs.insert(ConvertiblePair::new::<String, bool>());
        pairs.insert(ConvertiblePair::new::<String, f64>());
        let converter = ClosureGenericConverter::new(pairs, |s, _| Ok(s.to_string()));
        let s = format!("{converter:?}");
        assert!(s.contains("ClosureGenericConverter"));
        assert!(s.contains("3"), "should include pair count 3: {s}");
    }

    #[test]
    fn debug_format_for_empty_pairs_shows_zero() {
        let converter = ClosureGenericConverter::new(HashSet::new(), |s, _| Ok(s.to_string()));
        let s = format!("{converter:?}");
        assert!(s.contains("ClosureGenericConverter"));
        assert!(s.contains("0"), "should include pair count 0: {s}");
    }
}
