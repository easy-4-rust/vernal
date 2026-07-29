//! ConversionFailedException — Spring 风格的类型转换失败异常。
//!
//! 对应 Java 类：`org.springframework.core.convert.ConversionFailedException`。
//!
//! 当值从一种类型转换为另一种类型失败时抛出，包含源类型、目标类型与原始值。

use std::error::Error;
use std::fmt;

/// Spring 风格的类型转换失败异常。
///
/// 对应 Spring 的 `ConversionFailedException`。
///
/// 记录转换失败时的源类型描述、目标类型描述、原始值字符串表示以及可选原因。
#[derive(Debug)]
pub struct ConversionFailedException {
    /// 源类型描述（如 `"String"`、`"i32"`）。
    source_type: String,
    /// 目标类型描述。
    target_type: String,
    /// 原始值的字符串表示。
    value: String,
    /// 原始原因（可选）。
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl ConversionFailedException {
    /// 创建新的转换失败异常（无原因）。
    pub fn new(
        source_type: impl Into<String>,
        target_type: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            source_type: source_type.into(),
            target_type: target_type.into(),
            value: value.into(),
            cause: None,
        }
    }

    /// 创建带原因的转换失败异常。
    pub fn with_cause(
        source_type: impl Into<String>,
        target_type: impl Into<String>,
        value: impl Into<String>,
        cause: Box<dyn Error + Send + Sync>,
    ) -> Self {
        Self {
            source_type: source_type.into(),
            target_type: target_type.into(),
            value: value.into(),
            cause: Some(cause),
        }
    }

    /// 获取源类型描述。
    pub fn source_type(&self) -> &str {
        &self.source_type
    }

    /// 获取目标类型描述。
    pub fn target_type(&self) -> &str {
        &self.target_type
    }

    /// 获取原始值字符串表示。
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ConversionFailedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to convert from '{}' to '{}' for value '{}'",
            self.source_type, self.target_type, self.value
        )?;
        if let Some(ref cause) = self.cause {
            write!(f, "; caused by: {cause}")?;
        }
        Ok(())
    }
}

impl Error for ConversionFailedException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_ref()
            .map(|c| c.as_ref() as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyCause;
    impl fmt::Display for DummyCause {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("dummy cause")
        }
    }
    impl Error for DummyCause {}

    #[test]
    fn test_display() {
        let err = ConversionFailedException::new("String", "i32", "abc");
        assert_eq!(
            err.to_string(),
            "Failed to convert from 'String' to 'i32' for value 'abc'"
        );
        assert_eq!(err.source_type(), "String");
        assert_eq!(err.target_type(), "i32");
        assert_eq!(err.value(), "abc");
    }

    #[test]
    fn test_with_cause() {
        let err =
            ConversionFailedException::with_cause("String", "bool", "maybe", Box::new(DummyCause));
        assert!(err.source().is_some());
        assert!(err.to_string().contains("caused by: dummy cause"));
    }
}
