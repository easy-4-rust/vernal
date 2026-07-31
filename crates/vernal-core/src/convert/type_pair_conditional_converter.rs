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

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 source_type/target_type 返回 `TypeId` 而不是具体类型
    /// （对标 Spring `getConvertibleTypes()` 返回 `Set<ConvertiblePair>` 的运行时类型对查询）
    #[test]
    fn source_type_and_target_type_match_constructed_pair() {
        let converter = TypePairConditionalConverter::new::<String, i64>();
        let pair = ConvertiblePair::new::<String, i64>();
        assert_eq!(converter.source_type(), pair.source_type_id());
        assert_eq!(converter.target_type(), pair.target_type_id());
    }

    /// 同一对类型两次构造的 converter 应能互相 matches（对标 `equals` 语义）
    #[test]
    fn matches_returns_true_for_same_type_pair() {
        let converter = TypePairConditionalConverter::new::<String, i64>();
        let pair = ConvertiblePair::new::<String, i64>();
        assert!(converter.matches(&pair));
    }

    /// 源类型不同 → matches 返回 false
    #[test]
    fn matches_returns_false_when_source_type_differs() {
        let converter = TypePairConditionalConverter::new::<String, i64>();
        let pair = ConvertiblePair::new::<Vec<u8>, i64>();
        assert!(!converter.matches(&pair));
    }

    /// 目标类型不同 → matches 返回 false
    #[test]
    fn matches_returns_false_when_target_type_differs() {
        let converter = TypePairConditionalConverter::new::<String, i64>();
        let pair = ConvertiblePair::new::<String, f64>();
        assert!(!converter.matches(&pair));
    }

    /// 源和目标都不同 → matches 返回 false
    #[test]
    fn matches_returns_false_when_both_types_differ() {
        let converter = TypePairConditionalConverter::new::<String, i64>();
        let pair = ConvertiblePair::new::<bool, f64>();
        assert!(!converter.matches(&pair));
    }

    /// 不同 (S, T) 对构造的 converter 必须互不匹配（对标 Spring 不同 `ConditionalConverter` 互斥）
    #[test]
    fn two_distinct_converters_do_not_cross_match() {
        let a = TypePairConditionalConverter::new::<String, i64>();
        let b = TypePairConditionalConverter::new::<i64, String>();
        let pair_a = ConvertiblePair::new::<String, i64>();
        let pair_b = ConvertiblePair::new::<i64, String>();
        assert!(a.matches(&pair_a));
        assert!(!a.matches(&pair_b));
        assert!(b.matches(&pair_b));
        assert!(!b.matches(&pair_a));
    }

    /// 同一对类型多次构造应等价（`TypeId` 是稳定的）
    #[test]
    fn same_type_args_produce_equivalent_converters() {
        let a = TypePairConditionalConverter::new::<u32, String>();
        let b = TypePairConditionalConverter::new::<u32, String>();
        assert_eq!(a.source_type(), b.source_type());
        assert_eq!(a.target_type(), b.target_type());
    }
}
