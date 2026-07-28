//! MIME 邮件消息 — 对标 `MimeMailMessage`。

use super::mime_message::MimeMessage;
use crate::mail::MailMessage;

/// MIME 邮件消息。
///
/// 对标 Spring 的 `MimeMailMessage`，实现 `MailMessage` trait，
/// 将所有操作委托给 `MimeMessageHelper`。
pub struct MimeMailMessage {
    helper: MimeMessage,
}

impl MimeMailMessage {
    /// 从 MimeMessage 创建。
    pub fn new(message: MimeMessage) -> Self {
        Self { helper: message }
    }

    /// 获取内部 MIME 消息。
    pub fn mime_message(&self) -> &MimeMessage {
        &self.helper
    }

    /// 获取内部 MIME 消息的所有权。
    pub fn into_mime_message(self) -> MimeMessage {
        self.helper
    }

    /// 设置发件人。
    pub fn set_from(&mut self, from: &str) {
        self.helper.set_from(from);
    }
    /// 设置收件人。
    pub fn set_to(&mut self, to: &str) {
        self.helper.add_recipient(to);
    }
    /// 设置抄送。
    pub fn set_cc(&mut self, cc: &str) {
        self.helper.add_cc(cc);
    }
    /// 设置密送。
    pub fn set_bcc(&mut self, bcc: &str) {
        self.helper.add_bcc(bcc);
    }
    /// 设置主题。
    pub fn set_subject(&mut self, subject: &str) {
        self.helper.set_subject(subject);
    }
    /// 设置文本内容。
    pub fn set_text(&mut self, text: &str) {
        self.helper.set_text(text);
    }
}

impl MailMessage for MimeMailMessage {
    fn from(&self) -> Option<&str> {
        self.helper.from()
    }
    fn to(&self) -> &[String] {
        self.helper.to()
    }
    fn cc(&self) -> &[String] {
        &[]
    } // MimeMessage 内部存储
    fn bcc(&self) -> &[String] {
        &[]
    }
    fn subject(&self) -> Option<&str> {
        self.helper.subject()
    }
    fn text(&self) -> Option<&str> {
        self.helper.text_body()
    }
}
