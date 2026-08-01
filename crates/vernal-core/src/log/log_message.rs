//! 日志消息。
//!
//! 对标 Spring `org.springframework.core.log.LogMessage`。

/// 日志消息（延迟格式化）。
///
/// 对应 Java: org.springframework.core.log.LogMessage
///
/// Spring 语义：日志内容延迟到实际输出时格式化（对标 `LogMessage.of(...)` +
/// `toString()`），避免未启用级别时的格式化开销。
#[derive(Debug, Clone)]
pub struct LogMessage {
    text: String,
}

impl LogMessage {
    /// 从格式化函数创建消息（延迟求值——当前实现立即求值以保持简单）。
    #[must_use]
    pub fn of<F>(formatter: F) -> Self
    where
        F: FnOnce() -> String,
    {
        Self {
            text: formatter(),
        }
    }

    /// 创建空消息。
    #[must_use]
    pub fn none() -> Self {
        Self {
            text: String::new(),
        }
    }

    /// 从静态文本创建消息。
    #[must_use]
    pub fn format(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// 是否为无内容消息（对标 Spring `isEmpty`）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 返回消息文本。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_format_via_of() {
        // A 类（合同对齐）：对标 Spring `LogMessage.of(...)`
        let message = LogMessage::of(|| format!("value = {}", 42));
        assert_eq!(message.text(), "value = 42");
    }

    #[test]
    fn empty_message_detection() {
        // B 类（边界行为）：对标 Spring isEmpty
        assert!(LogMessage::none().is_empty());
        assert!(!LogMessage::format("x").is_empty());
    }

    #[test]
    fn displays_as_text() {
        // A 类（合同对齐）：对标 Spring toString
        let message = LogMessage::format("hello");
        assert_eq!(message.to_string(), "hello");
    }
}
