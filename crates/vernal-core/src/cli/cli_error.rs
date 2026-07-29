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
