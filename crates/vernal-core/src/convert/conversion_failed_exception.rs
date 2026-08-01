//! 转换失败异常。
//!
//! 对标 Spring `org.springframework.core.convert.ConversionFailedException`。

use std::fmt;

use crate::convert::ConversionError;

/// 转换失败异常。
///
/// 对应 Java: org.springframework.core.convert.ConversionFailedException
///
/// Spring 语义：携带源类型/目标类型/源值与失败原因（继承自
/// `ConversionException`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionFailedException {
    /// 源类型名。
    pub source_type: String,
    /// 目标类型名。
    pub target_type: String,
    /// 源值（字符串形式）。
    pub value: String,
    /// 失败原因。
    pub reason: String,
}

impl ConversionFailedException {
    /// 从 [`ConversionError`] 构造（对标 Spring 包装底层异常）。
    #[must_use]
    pub fn from_conversion_error(err: &ConversionError) -> Self {
        Self {
            source_type: "String".to_string(),
            target_type: err.target_type.to_string(),
            value: err.value.clone(),
            reason: err.reason.clone(),
        }
    }
}

impl fmt::Display for ConversionFailedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "转换失败: {} -> {} (值: {:?}): {}",
            self.source_type, self.target_type, self.value, self.reason
        )
    }
}

impl std::error::Error for ConversionFailedException {}

impl From<ConversionError> for ConversionFailedException {
    fn from(err: ConversionError) -> Self {
        Self::from_conversion_error(&err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_full_context() {
        // A 类（合同对齐）：对标 Spring 异常消息包含类型与值
        let err = ConversionFailedException {
            source_type: "String".to_string(),
            target_type: "i32".to_string(),
            value: "abc".to_string(),
            reason: "数字格式无效".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("i32"));
        assert!(s.contains("abc"));
        assert!(s.contains("数字格式无效"));
    }

    #[test]
    fn wraps_conversion_error() {
        // C 类（错误路径）：对标 Spring 底层异常包装
        let inner = ConversionError {
            value: "x".to_string(),
            target_type: "bool",
            reason: "期望 true/false".to_string(),
        };
        let err = ConversionFailedException::from(inner);
        assert_eq!(err.target_type, "bool");
        assert_eq!(err.value, "x");
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ConversionFailedException>();
    }
}
