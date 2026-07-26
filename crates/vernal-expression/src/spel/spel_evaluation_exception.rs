//! SpEL 求值异常。
//!
//! 对标 Spring 的 `SpelEvaluationException`。

use super::spel_message::SpelMessage;

/// SpEL 求值异常。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelEvaluationException`。
#[derive(Debug, Clone)]
pub struct SpelEvaluationException {
    /// 错误消息码
    message_code: SpelMessage,
    /// 表达式位置
    position: Option<usize>,
}

impl SpelEvaluationException {
    /// 创建 SpEL 求值异常。
    #[must_use]
    pub fn new(message_code: SpelMessage, position: Option<usize>) -> Self {
        Self {
            message_code,
            position,
        }
    }

    /// 获取错误消息码。
    #[must_use]
    pub fn message_code(&self) -> SpelMessage {
        self.message_code
    }

    /// 获取表达式位置。
    #[must_use]
    pub fn position(&self) -> Option<usize> {
        self.position
    }
}

impl std::fmt::Display for SpelEvaluationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.position {
            Some(pos) => write!(
                f,
                "[{}] {} @ position {}",
                self.message_code.code(),
                self.message_code.default_message(),
                pos
            ),
            None => write!(
                f,
                "[{}] {}",
                self.message_code.code(),
                self.message_code.default_message()
            ),
        }
    }
}

impl std::error::Error for SpelEvaluationException {}
