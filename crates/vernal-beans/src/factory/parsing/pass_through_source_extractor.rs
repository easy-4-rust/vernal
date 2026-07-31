//! PassThroughSourceExtractor — 对应 Spring PassThroughSourceExtractor。
//!
//! 直接传递原始元数据对象的源提取器。将传入的 source 对象原样返回，
//! 适用于需要保留完整源信息（如 XML DOM 节点）的调试场景。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.PassThroughSourceExtractor`。

use std::any::Any;

use super::source_extractor::SourceExtractor;

/// 直通源提取器。
///
/// 对应 Spring 的 `PassThroughSourceExtractor`。
///
/// 将传入的元数据对象原样返回（装箱后作为 `Box<dyn Any>` 返回）。
/// 适用于调试场景，需要在错误报告中保留完整的源上下文。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::pass_through_source_extractor::PassThroughSourceExtractor;
/// use vernal_beans::factory::parsing::source_extractor::SourceExtractor;
///
/// let extractor = PassThroughSourceExtractor::new();
/// let data = String::from("<bean id='myBean'/>");
/// let result = extractor.extract_source(&data).unwrap();
/// assert_eq!(result.downcast_ref::<String>().unwrap(), "<bean id='myBean'/>");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PassThroughSourceExtractor;

impl PassThroughSourceExtractor {
    /// 创建一个新的直通源提取器。
    pub fn new() -> Self {
        Self
    }
}

impl SourceExtractor for PassThroughSourceExtractor {
    fn extract_source(&self, source: &dyn Any) -> Option<Box<dyn Any>> {
        // 如果 source 是 &String，克隆后返回
        if let Some(s) = source.downcast_ref::<String>() {
            return Some(Box::new(s.clone()));
        }
        // 如果 source 是 &str，转为 String 返回
        if let Some(s) = source.downcast_ref::<&str>() {
            return Some(Box::new(s.to_string()));
        }
        // 如果 source 是 &i64 等 Copy 类型，克隆值
        if let Some(v) = source.downcast_ref::<i64>() {
            return Some(Box::new(*v));
        }
        if let Some(v) = source.downcast_ref::<i32>() {
            return Some(Box::new(*v));
        }
        if let Some(v) = source.downcast_ref::<bool>() {
            return Some(Box::new(*v));
        }
        // 对于不支持 Clone 的复杂类型，返回 None
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_through_string() {
        let extractor = PassThroughSourceExtractor::new();
        let data = String::from("<bean/>");
        let result = extractor.extract_source(&data).unwrap();
        assert_eq!(result.downcast_ref::<String>().unwrap(), "<bean/>");
    }

    #[test]
    fn test_pass_through_i32() {
        let extractor = PassThroughSourceExtractor::new();
        let data = 42i32;
        let result = extractor.extract_source(&data).unwrap();
        assert_eq!(result.downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn test_pass_through_bool() {
        let extractor = PassThroughSourceExtractor::new();
        let data = true;
        let result = extractor.extract_source(&data).unwrap();
        assert_eq!(result.downcast_ref::<bool>().unwrap(), &true);
    }
}
