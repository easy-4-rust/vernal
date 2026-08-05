//! SpelEvaluationException — 求值阶段错误。
//!
//! 对标 Spring `org.springframework.expression.spel.SpelEvaluationException`，
//! 使用 `thiserror` 派生 `std::error::Error`。

use thiserror::Error;

use super::spel_message::SpelMessage;

/// SpEL 求值异常（对标 Spring `SpelEvaluationException`）。
#[derive(Debug, Error)]
pub struct SpelEvaluationException {
    /// 错误码。
    pub code: SpelMessage,
    /// 插入占位符。
    pub inserts: Vec<String>,
    /// 错误位置。
    pub position: Option<usize>,
    /// 渲染消息。
    message: String,
}

impl SpelEvaluationException {
    /// 创建求值异常（无位置）。
    pub fn new(code: SpelMessage, inserts: &[&str]) -> Self {
        let message = code.format_message(inserts);
        Self {
            code,
            inserts: inserts.iter().map(|s| s.to_string()).collect(),
            position: None,
            message,
        }
    }

    /// 创建求值异常（带位置）。
    pub fn at(code: SpelMessage, position: usize, inserts: &[&str]) -> Self {
        let message = code.format_message(inserts);
        Self {
            code,
            inserts: inserts.iter().map(|s| s.to_string()).collect(),
            position: Some(position),
            message,
        }
    }

    /// 设置位置（对标 Spring `setPosition`，某些构造场景需要后置设置）。
    pub fn set_position(&mut self, position: usize) {
        self.position = Some(position);
    }

    /// 获取错误码（对标 Spring `getMessageCode`）。
    #[must_use]
    pub fn message_code(&self) -> SpelMessage {
        self.code
    }

    /// 获取插入（对标 Spring `getInserts`）。
    #[must_use]
    pub fn inserts(&self) -> &[String] {
        &self.inserts
    }

    /// 获取位置。
    #[must_use]
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    /// 简易消息（`EL{code}E: <formatted>`）。
    #[must_use]
    pub fn simple_message(&self) -> &str {
        &self.message
    }

    /// 渲染消息。
    #[must_use]
    pub fn formatted_message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for SpelEvaluationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.position {
            Some(pos) => write!(f, "{} @position {}", self.simple_message(), pos),
            None => f.write_str(self.simple_message()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_at() {
        let ex = SpelEvaluationException::at(
            SpelMessage::PropertyOrFieldNotReadable,
            12,
            &["foo", "Object"],
        );
        assert_eq!(ex.position(), Some(12));
        assert!(ex.simple_message().contains("'foo'"));
    }

    #[test]
    fn new_no_position() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        assert!(ex.position().is_none());
    }

    #[test]
    fn set_position() {
        let mut ex =
            SpelEvaluationException::new(SpelMessage::NotComparable, &["Integer", "String"]);
        ex.set_position(7);
        assert_eq!(ex.position(), Some(7));
    }
}
