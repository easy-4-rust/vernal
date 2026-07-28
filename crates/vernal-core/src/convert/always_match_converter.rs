//! 始终匹配的条件转换器（用于注册 fallback 转换器）。
//!
//! 对标 Spring `ConditionalConverter` 实现中的无条件匹配版本。

use super::conditional_converter::ConditionalConverter;
use super::convertible_pair::ConvertiblePair;

/// 始终匹配的条件转换器。
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysMatchConverter;

impl AlwaysMatchConverter {
    /// 创建新的始终匹配转换器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ConditionalConverter for AlwaysMatchConverter {
    fn matches(&self, _pair: &ConvertiblePair) -> bool {
        true
    }
}
