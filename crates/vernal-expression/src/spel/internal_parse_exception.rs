//! InternalParseException — 解析器内部流转异常。
//!
//! 对标 Spring `org.springframework.expression.spel.InternalParseException`。
//! 该异常是 `RuntimeException`，目的是让深层的 `eatXxx` / `maybeEatXxx` 抛出 SpelParseException
//! 时不用跨越复杂控制流；顶层 `doParseExpression` 抓住后 unwrap 出原始 SpelParseException 重抛。

use super::spel_message::SpelMessage;
use super::spel_parse_exception::SpelParseException;

/// 内部解析异常（控制流异常，不应暴露给最终用户）。
///
/// 对标 Spring `InternalParseException(SpelParseException)`。
#[derive(Clone)]
pub struct InternalParseException {
    /// 携带的 SpelParseException。
    cause: SpelParseException,
}

impl std::fmt::Debug for InternalParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalParseException")
            .field("cause", &self.cause)
            .finish()
    }
}

impl InternalParseException {
    /// 通过 SpelParseException 包装。
    #[must_use]
    pub fn wrap(cause: SpelParseException) -> Self {
        Self { cause }
    }

    /// 通过代码与 inserts 快捷构造。
    pub fn new(
        expression: impl Into<String>,
        position: usize,
        code: SpelMessage,
        inserts: &[&str],
    ) -> Self {
        Self {
            cause: SpelParseException::new(expression, position, code, inserts),
        }
    }

    /// 取出内部的 SpelParseException。
    #[must_use]
    pub fn into_parse_exception(self) -> SpelParseException {
        self.cause
    }

    /// 借用内部的 SpelParseException。
    #[must_use]
    pub fn parse_exception(&self) -> &SpelParseException {
        &self.cause
    }
}

impl std::fmt::Display for InternalParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.cause, f)
    }
}

impl std::error::Error for InternalParseException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
