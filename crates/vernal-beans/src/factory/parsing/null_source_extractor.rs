//! NullSourceExtractor — 对应 Spring NullSourceExtractor。
//!
//! 始终返回 `None` 的源提取器。当不需要保留 Bean 定义的源信息时使用，
//! 可以减少内存占用。这是 Spring 的默认 SourceExtractor 实现。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.NullSourceExtractor`。

use std::any::Any;

use super::source_extractor::SourceExtractor;

/// 空源提取器。
///
/// 对应 Spring 的 `NullSourceExtractor`。
///
/// 所有提取请求都返回 `None`，不保留任何源信息。适用于不需要
/// 精确错误定位的场景，可以减少内存开销。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::null_source_extractor::NullSourceExtractor;
/// use vernal_beans::factory::parsing::source_extractor::SourceExtractor;
///
/// let extractor = NullSourceExtractor::new();
/// let source = extractor.extract_source(&"some xml element");
/// assert!(source.is_none());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NullSourceExtractor;

impl NullSourceExtractor {
    /// 创建一个新的空源提取器。
    pub fn new() -> Self {
        Self
    }
}

impl SourceExtractor for NullSourceExtractor {
    fn extract_source(&self, _source: &dyn Any) -> Option<Box<dyn Any>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_returns_none() {
        let extractor = NullSourceExtractor::new();
        assert!(extractor.extract_source(&42i32).is_none());
        assert!(extractor.extract_source(&"hello").is_none());
        assert!(extractor.extract_source(&true).is_none());
    }

    #[test]
    fn test_default_construction() {
        let extractor = NullSourceExtractor;
        assert!(extractor.extract_source(&()).is_none());
    }

    #[test]
    fn test_multiple_calls_consistent() {
        let extractor = NullSourceExtractor::new();
        let data = vec![1, 2, 3];
        // 验证多次调用都返回 None
        for _ in 0..5 {
            assert!(extractor.extract_source(&data).is_none());
        }
    }
}
