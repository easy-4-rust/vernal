//! SpEL 解析异常。
//!
//! 对标 Spring 的 `SpelParseException`。

use super::spel_message::SpelMessage;

/// SpEL 解析异常。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelParseException`。
#[derive(Debug, Clone)]
pub struct SpelParseException {
    /// 表达式字符串
    expression: String,
    /// 错误位置
    position: Option<usize>,
    /// 错误消息码
    message_code: SpelMessage,
    /// 错误消息参数
    inserts: Vec<String>,
}

impl SpelParseException {
    /// 创建 SpEL 解析异常。
    #[must_use]
    pub fn new(
        expression: String,
        position: Option<usize>,
        message_code: SpelMessage,
        inserts: Vec<String>,
    ) -> Self {
        Self {
            expression,
            position,
            message_code,
            inserts,
        }
    }

    /// 获取错误消息码。
    #[must_use]
    pub fn message_code(&self) -> SpelMessage {
        self.message_code
    }

    /// 获取错误消息参数。
    #[must_use]
    pub fn inserts(&self) -> &[String] {
        &self.inserts
    }
}

impl std::fmt::Display for SpelParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}",
            self.message_code.code(),
            self.message_code.default_message()
        )
    }
}

impl std::error::Error for SpelParseException {}
