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

#[cfg(test)]
mod tests {
    use super::*;

    /// 对标 Spring `NoOpConditionalConverter.matches()`: 永远返回 false
    #[test]
    fn matches_returns_false_for_any_pair() {
        let converter = NeverMatchConverter::new();
        assert!(!converter.matches(&ConvertiblePair::new::<String, i64>()));
        assert!(!converter.matches(&ConvertiblePair::new::<i32, bool>()));
        assert!(!converter.matches(&ConvertiblePair::new::<Vec<u8>, String>()));
    }

    /// Default 实例与 `new()` 等价
    #[test]
    fn default_equivalent_to_new() {
        let from_new = NeverMatchConverter::new();
        let from_default = NeverMatchConverter;
        // 都不匹配任何 pair
        let pair = ConvertiblePair::new::<String, i64>();
        assert!(!from_new.matches(&pair));
        assert!(!from_default.matches(&pair));
    }

    /// Copy 语义: 实例可被复制且保持行为
    #[test]
    fn is_copy_semantics() {
        let converter = NeverMatchConverter::new();
        let copy1 = converter; // Copy: 不移动
        let copy2 = converter; // 仍然可用, 因为 Copy
        let pair = ConvertiblePair::new::<String, i64>();
        assert!(!copy1.matches(&pair));
        assert!(!copy2.matches(&pair));
    }

    /// Debug 应可派生
    #[test]
    fn supports_debug_format() {
        let converter = NeverMatchConverter::new();
        let s = format!("{converter:?}");
        assert!(s.contains("NeverMatchConverter"));
    }
}
