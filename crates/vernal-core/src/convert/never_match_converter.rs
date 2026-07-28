//! 从不匹配的条件转换器（用于禁用转换器）。
//!
//! 对标 Spring `ConditionalConverter` 实现中的无条件不匹配版本。

use super::conditional_converter::ConditionalConverter;
use super::convertible_pair::ConvertiblePair;

/// 从不匹配的条件转换器。
#[derive(Debug, Clone, Copy, Default)]
pub struct NeverMatchConverter;

impl NeverMatchConverter {
    /// 创建新的从不匹配转换器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ConditionalConverter for NeverMatchConverter {
    fn matches(&self, _pair: &ConvertiblePair) -> bool {
        false
    }
}
