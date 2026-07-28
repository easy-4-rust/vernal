//! SpelParseException — 解析阶段错误。
//!
//! 对标 Spring `org.springframework.expression.spel.SpelParseException`，
//! 使用 `thiserror` 派生 `std::error::Error`。

use thiserror::Error;

use super::spel_message::SpelMessage;

/// SpEL 解析异常（对标 Spring `SpelParseException`）。
#[derive(Debug, Clone, Error)]
pub struct SpelParseException {
    /// 错误码。
    pub code: SpelMessage,
    /// 插入占位符字符串。
    pub inserts: Vec<String>,
    /// 表达式原始字符串。
    pub expression_string: Option<String>,
    /// 错误位置（字节偏移）。
    pub position: Option<usize>,
    /// 完整渲染消息（`EL{code}E: <formatted>`）。
    message: String,
}

impl SpelParseException {
    /// 创建带表达式与位置的解析异常。
    pub fn new(
        expression_string: impl Into<String>,
        position: usize,
        code: SpelMessage,
        inserts: &[&str],
    ) -> Self {
        let expr_str = expression_string.into();
        let inserts_vec: Vec<String> = inserts.iter().map(|s| s.to_string()).collect();
        let message = code.format_message(inserts);
        Self {
            code,
            inserts: inserts_vec,
            expression_string: Some(expr_str),
            position: Some(position),
            message,
        }
    }

    /// 创建无位置异常。
    pub fn at_unknown(code: SpelMessage, inserts: &[&str]) -> Self {
        let message = code.format_message(inserts);
        Self {
            code,
            inserts: inserts.iter().map(|s| s.to_string()).collect(),
            expression_string: None,
            position: None,
            message,
        }
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

    /// 获取渲染消息。
    #[must_use]
    pub fn formatted_message(&self) -> &str {
        &self.message
    }

    /// 简易消息（`super.getMessage()`，对标 Spring `getSimpleMessage`）。
    #[must_use]
    pub fn simple_message(&self) -> &str {
        &self.message
    }

    /// 详细消息：`Expression [{expr}] @{pos}: {simple}`。
    #[must_use]
    pub fn detailed_message(&self) -> String {
        match (&self.expression_string, self.position) {
            (Some(expr), Some(pos)) => {
                format!("Expression [{expr}] @{pos}: {}", self.simple_message())
            }
            _ => self.simple_message().to_string(),
        }
    }
}

impl std::fmt::Display for SpelParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detailed_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_render() {
        let ex = SpelParseException::new(
            "1 + )",
            4,
            SpelMessage::NotExpectedToken,
            &["rparen", "rparen"],
        );
        assert_eq!(ex.code, SpelMessage::NotExpectedToken);
        assert_eq!(ex.position, Some(4));
        assert!(ex.formatted_message().starts_with("EL1043E:"));
        assert_eq!(ex.inserts().len(), 2);
    }

    #[test]
    fn at_unknown() {
        let ex = SpelParseException::at_unknown(SpelMessage::Ood, &[]);
        assert_eq!(ex.code, SpelMessage::Ood);
        assert!(ex.position.is_none());
        assert!(ex.expression_string.is_none());
    }

    #[test]
    fn formatted_substitutes() {
        let ex = SpelParseException::new(
            "foo",
            0,
            SpelMessage::TypeConversionError,
            &["Integer", "String"],
        );
        let s = ex.formatted_message();
        assert!(s.contains("from Integer to String"));
    }
}
