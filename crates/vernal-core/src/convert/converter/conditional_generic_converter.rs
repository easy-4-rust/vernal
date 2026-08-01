//! 条件通用转换器 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConditionalGenericConverter`。

use super::{ConditionalConverter, GenericConverter};

/// 条件通用转换器 trait。
///
/// 对应 Java: org.springframework.core.convert.converter.ConditionalGenericConverter
///
/// Spring 语义：`ConditionalConverter` 与 `GenericConverter` 的组合——先按
/// 源/目标类型对做条件匹配，命中后才执行多对多转换。
pub trait ConditionalGenericConverter: ConditionalConverter + GenericConverter {}

impl<T: ConditionalConverter + GenericConverter> ConditionalGenericConverter for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::converter::ConvertiblePair;
    use std::collections::HashSet;

    struct NoopConditionalGeneric;

    impl ConditionalConverter for NoopConditionalGeneric {
        fn matches(&self, _pair: &ConvertiblePair) -> bool {
            false
        }
    }

    impl GenericConverter for NoopConditionalGeneric {
        fn convertible_types(&self) -> HashSet<ConvertiblePair> {
            HashSet::new()
        }

        fn convert(
            &self,
            source: &str,
            _target_type: std::any::TypeId,
        ) -> Result<String, crate::convert::ConversionError> {
            Ok(source.to_string())
        }
    }

    #[test]
    fn blanket_impl_applies() {
        // D 类（重构安全）：同时实现两个契约的类型自动获得合并契约
        fn assert_conditional_generic<T: ConditionalGenericConverter>() {}
        assert_conditional_generic::<NoopConditionalGeneric>();
    }
}
