//! 占位符解析异常。
//!
//! 对标 Spring `org.springframework.util.PlaceholderResolutionException`
//! （以及 `PlaceholderResolver` 相关的异常体系）。
//!
//! 当前 `PropertyPlaceholderHelper` 使用的 `PlaceholderError` 枚举是轻量错误类型；
//! 本模块提供对标 Spring 异常的**结构化错误类型**，包含占位符名称和原因链，
//! 适合在需要 Spring 兼容诊断的场景使用。

use std::error::Error;
use std::fmt;

/// 占位符解析异常。
///
/// 对标 Spring `PlaceholderResolutionException`。
///
/// 携带触发错误的占位符名称（如 `${my.key}`）和底层原因。
#[derive(Debug)]
pub struct PlaceholderResolutionException {
    /// 触发异常的占位符名称（含前缀后缀，如 `${my.key}`）。
    placeholder: String,
    /// 脱敏的人类可读消息。
    message: String,
    /// 底层原因（不暴露原始值，防止凭证泄漏）。
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl PlaceholderResolutionException {
    /// 创建新的占位符解析异常。
    ///
    /// # 参数
    ///
    /// - `placeholder`：占位符名称（如 `${my.key}`）
    /// - `message`：脱敏消息
    #[must_use]
    pub fn new(placeholder: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            message: message.into(),
            source: None,
        }
    }

    /// 创建带底层原因的异常。
    #[must_use]
    pub fn with_source(
        placeholder: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn Error + Send + Sync + 'static>,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    /// 返回触发异常的占位符名称。
    ///
    /// 对标 Spring `getPlaceholder()`。
    #[must_use]
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// 返回脱敏消息（不含占位符值）。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PlaceholderResolutionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Could not resolve placeholder '{}' — {}",
            self.placeholder, self.message
        )
    }
}

impl Error for PlaceholderResolutionException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn Error))
    }
}

/// 从 [`super::property_placeholder_helper::PlaceholderError`] 转换。
impl From<&super::property_placeholder_helper::PlaceholderError>
    for PlaceholderResolutionException
{
    fn from(err: &super::property_placeholder_helper::PlaceholderError) -> Self {
        match err {
            super::property_placeholder_helper::PlaceholderError::UnresolvedPlaceholder {
                placeholder,
            } => Self::new(placeholder.clone(), "unresolved placeholder"),
            super::property_placeholder_helper::PlaceholderError::InvalidSyntax {
                input,
                reason,
            } => Self::new(input.clone(), format!("invalid syntax: {reason}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_basic() {
        let err = PlaceholderResolutionException::new("${my.key}", "not found");
        assert_eq!(err.placeholder(), "${my.key}");
        assert_eq!(err.message(), "not found");
        assert!(err.source().is_none());
    }

    #[test]
    fn display_contains_placeholder() {
        let err = PlaceholderResolutionException::new("${db.password}", "unresolved");
        let s = format!("{err}");
        assert!(s.contains("${db.password}"));
        assert!(s.contains("unresolved"));
    }

    #[test]
    fn with_source_has_cause() {
        let io_err = std::io::Error::other("disk read failed");
        let err =
            PlaceholderResolutionException::with_source("${config}", "io error", Box::new(io_err));
        assert!(err.source().is_some());
    }

    #[test]
    fn from_placeholder_error_unresolved() {
        let pe =
            super::super::property_placeholder_helper::PlaceholderError::UnresolvedPlaceholder {
                placeholder: "${missing}".to_string(),
            };
        let pre: PlaceholderResolutionException = (&pe).into();
        assert_eq!(pre.placeholder(), "${missing}");
        assert!(pre.message().contains("unresolved"));
    }

    #[test]
    fn from_placeholder_error_invalid_syntax() {
        let pe = super::super::property_placeholder_helper::PlaceholderError::InvalidSyntax {
            input: "${bad".to_string(),
            reason: "unclosed".to_string(),
        };
        let pre: PlaceholderResolutionException = (&pe).into();
        assert_eq!(pre.placeholder(), "${bad");
        assert!(pre.message().contains("invalid syntax"));
        assert!(pre.message().contains("unclosed"));
    }
}
