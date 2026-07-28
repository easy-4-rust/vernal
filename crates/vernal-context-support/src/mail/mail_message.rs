//! 邮件消息 trait — 对标 `org.springframework.mail.MailMessage`。

/// 邮件消息 trait。
///
/// 对标 Spring 的 `MailMessage`，提供邮件消息的通用接口。
pub trait MailMessage: Send + Sync {
    /// 获取发件人。
    fn from(&self) -> Option<&str>;
    /// 获取收件人。
    fn to(&self) -> &[String];
    /// 获取抄送。
    fn cc(&self) -> &[String];
    /// 获取密送。
    fn bcc(&self) -> &[String];
    /// 获取主题。
    fn subject(&self) -> Option<&str>;
    /// 获取文本内容。
    fn text(&self) -> Option<&str>;
}
