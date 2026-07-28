//! 邮件发送器 trait — 对标 `org.springframework.mail.MailSender`。

use super::mail_exception::MailError;
use super::mail_message::MailMessage;

/// 邮件发送器 trait。
///
/// 对标 Spring 的 `MailSender`，提供发送邮件的接口。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `send(SimpleMailMessage)` | `send_message()` | 发送简单邮件 |
pub trait MailSender: Send + Sync {
    /// 发送简单邮件。
    fn send_message(&self, message: &dyn MailMessage) -> Result<(), MailError>;
}
