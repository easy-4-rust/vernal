//! 简单邮件消息 — 对标 `org.springframework.mail.SimpleMailMessage`。

use super::mail_message::MailMessage;

/// 简单邮件消息。
///
/// 对标 Spring 的 `SimpleMailMessage`，包含 from/to/cc/bcc/subject/text 字段。
#[derive(Debug, Clone, Default)]
pub struct SimpleMailMessage {
    from: Option<String>,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: Option<String>,
    text: Option<String>,
}

impl SimpleMailMessage {
    /// 创建空的邮件消息。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置发件人。
    pub fn set_from(&mut self, from: impl Into<String>) {
        self.from = Some(from.into());
    }

    /// 设置收件人。
    pub fn set_to(&mut self, to: impl Into<String>) {
        self.to = vec![to.into()];
    }

    /// 添加收件人。
    pub fn add_to(&mut self, to: impl Into<String>) {
        self.to.push(to.into());
    }

    /// 设置抄送。
    pub fn set_cc(&mut self, cc: impl Into<String>) {
        self.cc = vec![cc.into()];
    }

    /// 设置密送。
    pub fn set_bcc(&mut self, bcc: impl Into<String>) {
        self.bcc = vec![bcc.into()];
    }

    /// 设置主题。
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.subject = Some(subject.into());
    }

    /// 设置文本内容。
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = Some(text.into());
    }
}

impl MailMessage for SimpleMailMessage {
    fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    fn to(&self) -> &[String] {
        &self.to
    }

    fn cc(&self) -> &[String] {
        &self.cc
    }

    fn bcc(&self) -> &[String] {
        &self.bcc
    }

    fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mail_message() {
        let mut msg = SimpleMailMessage::new();
        msg.set_from("sender@example.com");
        msg.set_to("recipient@example.com");
        msg.set_subject("Test Subject");
        msg.set_text("Hello, World!");

        assert_eq!(msg.from(), Some("sender@example.com"));
        assert_eq!(msg.to(), &["recipient@example.com"]);
        assert_eq!(msg.subject(), Some("Test Subject"));
        assert_eq!(msg.text(), Some("Hello, World!"));
    }

    #[test]
    fn test_multiple_recipients() {
        let mut msg = SimpleMailMessage::new();
        msg.add_to("user1@example.com");
        msg.add_to("user2@example.com");

        assert_eq!(msg.to().len(), 2);
    }
}
