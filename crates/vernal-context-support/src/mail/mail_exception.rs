//! 邮件异常类型 — 对标 `org.springframework.mail.MailException`。

/// 邮件错误类型。
///
/// 对标 Spring 的 `MailException` 及其子类。
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    /// 邮件解析失败。
    #[error("邮件解析失败：{0}")]
    ParseError(String),

    /// 邮件准备失败。
    #[error("邮件准备失败：{0}")]
    PreparationError(String),

    /// 邮件发送失败。
    #[error("邮件发送失败：{0}")]
    SendError(String),

    /// 邮件认证失败。
    #[error("邮件认证失败：{0}")]
    AuthenticationError(String),

    /// 邮件操作失败。
    #[error("邮件操作失败：{0}")]
    OperationError(String),
}
