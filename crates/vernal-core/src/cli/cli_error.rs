//! CLI 错误。
//!
//! 对标 Spring `org.springframework.boot.cli.parser.exceptions`。

/// CLI 错误。
///
/// 对应 Java: 上层 bootloader 框架 CLI 解析错误
#[derive(Debug, Clone)]
pub struct CliError {
    /// 错误消息
    pub message: String,
}

impl CliError {
    /// 创建新的 CLI 错误。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CLI 解析错误: {}", self.message)
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_str() {
        // 对标 Spring: 接受 &str 消息
        let err = CliError::new("invalid arg");
        assert_eq!(err.message, "invalid arg");
    }

    #[test]
    fn new_from_string() {
        let err = CliError::new(String::from("owned"));
        assert_eq!(err.message, "owned");
    }

    #[test]
    fn display_includes_message() {
        let err = CliError::new("flag missing");
        assert_eq!(format!("{err}"), "CLI 解析错误: flag missing");
    }

    #[test]
    fn error_trait_source_is_none() {
        use std::error::Error as _;
        let err = CliError::new("x");
        assert!(err.source().is_none());
    }

    #[test]
    fn clone_preserves_message() {
        let original = CliError::new("original message");
        let cloned = original.clone();
        assert_eq!(original.message, cloned.message);
        assert_eq!(format!("{original}"), format!("{cloned}"));
    }

    #[test]
    fn debug_format_includes_message() {
        let err = CliError::new("debug test");
        let dbg = format!("{err:?}");
        assert!(dbg.contains("CliError"));
        assert!(dbg.contains("debug test"));
    }
}
