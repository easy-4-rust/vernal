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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_default_converter() {
        // 对标 Spring: new() 应返回默认（无条件匹配）的转换器
        let c = AlwaysMatchConverter::new();
        let pair = ConvertiblePair::new::<String, i64>();
        assert!(c.matches(&pair), "AlwaysMatchConverter 应匹配所有类型对");
    }

    #[test]
    fn default_works_like_new() {
        // 对标 Spring: Default trait 应提供等价于 new() 的实例
        let c = AlwaysMatchConverter::default();
        let pair = ConvertiblePair::new::<String, i64>();
        assert!(c.matches(&pair));
    }

    #[test]
    fn debug_format_is_struct_name() {
        let c = AlwaysMatchConverter::new();
        assert_eq!(format!("{:?}", c), "AlwaysMatchConverter");
    }

    #[test]
    fn copy_semantics_preserve_behavior() {
        let c1 = AlwaysMatchConverter::new();
        let c2 = c1; // Copy
        let pair = ConvertiblePair::new::<i32, String>();
        assert!(c1.matches(&pair), "original still works after copy");
        assert!(c2.matches(&pair), "copy also works");
    }

    #[test]
    fn clone_equals_original() {
        let c1 = AlwaysMatchConverter::new();
        let c2 = c1.clone();
        let pair = ConvertiblePair::new::<bool, i32>();
        assert_eq!(c1.matches(&pair), c2.matches(&pair));
    }
}
