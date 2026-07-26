//! 内部解析异常。
//!
//! 对标 Spring 的 `InternalParseException`：在解析器内部流转，最终转换为 SpelParseException。

use crate::parse_exception::ParseException;

/// 内部解析异常。
///
/// 在解析器内部流转，最终在顶层的 parseExpression 方法中转换为 ParseException。
/// 对标 Spring 的 `org.springframework.expression.spel.InternalParseException`。
#[derive(Debug, Clone)]
pub struct InternalParseException {
    message: String,
    position: usize,
}

impl InternalParseException {
    /// 创建内部解析异常。
    #[must_use]
    pub fn new(message: String, position: usize) -> Self {
        Self { message, position }
    }

    /// 转换为公共 ParseException。
    #[must_use]
    pub fn to_parse_exception(&self, expression: String) -> ParseException {
        ParseException::new(expression, Some(self.position), self.message.clone())
    }
}

impl std::fmt::Display for InternalParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for InternalParseException {}
