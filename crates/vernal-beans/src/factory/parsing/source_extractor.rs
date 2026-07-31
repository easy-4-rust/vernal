//! SourceExtractor — 对应 Spring beans.factory.parsing.SourceExtractor。
//!
//! 从 Bean 定义元数据中提取源对象。在解析过程中，源对象描述了 Bean
//! 定义的来源（例如 XML 元素、注解等），用于错误报告和调试。不同的
//! SourceExtractor 实现可以选择提取完整的源信息或返回 null。
//!
//! 对应 Java 接口：`org.springframework.beans.factory.parsing.SourceExtractor`。

use std::any::Any;

/// 源对象提取器。
///
/// 对应 Spring 的 `SourceExtractor`。
///
/// 在 Bean 定义解析过程中，SourceExtractor 负责从原始元数据（如
/// XML Element、注解等）中提取出用于错误报告的源对象。源对象的类型
/// 由具体实现决定。
///
/// ## 标准实现
///
/// - [`super::null_source_extractor::NullSourceExtractor`] — 始终返回
///   `None`，不保留源信息
/// - [`super::pass_through_source_extractor::PassThroughSourceExtractor`]
///   — 直接传递原始元数据对象
pub trait SourceExtractor: Send + Sync {
    /// 从原始元数据中提取源对象。
    ///
    /// # 参数
    /// - `source` — 原始元数据（如 XML Element 的 erased 引用）
    ///
    /// # 返回
    /// 提取出的源对象；返回 `None` 表示不保留源信息。
    fn extract_source(&self, source: &dyn Any) -> Option<Box<dyn Any>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的记录型 SourceExtractor。
    struct RecordingExtractor {
        call_count: std::sync::Mutex<usize>,
    }

    impl RecordingExtractor {
        fn new() -> Self {
            Self {
                call_count: std::sync::Mutex::new(0),
            }
        }

        fn call_count(&self) -> usize {
            *self.call_count.lock().unwrap()
        }
    }

    impl SourceExtractor for RecordingExtractor {
        fn extract_source(&self, _source: &dyn Any) -> Option<Box<dyn Any>> {
            *self.call_count.lock().unwrap() += 1;
            Some(Box::new("extracted".to_string()))
        }
    }

    #[test]
    fn test_extract_source_returns_value() {
        let extractor = RecordingExtractor::new();
        let data = 42i32;
        let result = extractor.extract_source(&data);
        assert!(result.is_some());
        assert_eq!(extractor.call_count(), 1);
    }

    #[test]
    fn test_extract_source_called_multiple_times() {
        let extractor = RecordingExtractor::new();
        let data = "hello";
        let _ = extractor.extract_source(&data);
        let _ = extractor.extract_source(&data);
        let _ = extractor.extract_source(&data);
        assert_eq!(extractor.call_count(), 3);
    }
}
