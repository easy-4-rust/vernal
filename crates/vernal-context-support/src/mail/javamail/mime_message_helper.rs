//! MIME 消息助手 — 对标 `org.springframework.mail.javamail.MimeMessageHelper`。

use super::mime_message::MimeMessage;

/// MIME 消息助手。
///
/// 对标 Spring 的 `MimeMessageHelper`，提供便捷的 MIME 消息构建方法。
pub struct MimeMessageHelper {
    message: MimeMessage,
}

impl MimeMessageHelper {
    /// 创建 MIME 消息助手。
    pub fn new(message: MimeMessage) -> Self {
        Self { message }
    }

    /// 设置发件人。
    pub fn set_from(&mut self, from: impl Into<String>) {
        self.message.set_from(from);
    }

    /// 添加收件人。
    pub fn add_to(&mut self, to: impl Into<String>) {
        self.message.add_recipient(to);
    }

    /// 添加抄送。
    pub fn add_cc(&mut self, cc: impl Into<String>) {
        self.message.add_cc(cc);
    }

    /// 添加密送。
    pub fn add_bcc(&mut self, bcc: impl Into<String>) {
        self.message.add_bcc(bcc);
    }

    /// 设置主题。
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.message.set_subject(subject);
    }

    /// 设置文本内容。
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.message.set_text(text);
    }

    /// 设置 HTML 内容。
    pub fn set_html(&mut self, html: impl Into<String>) {
        self.message.set_html(html);
    }

    /// 获取内部 MIME 消息。
    pub fn mime_message(&self) -> &MimeMessage {
        &self.message
    }

    /// 获取内部 MIME 消息的所有权。
    pub fn into_mime_message(self) -> MimeMessage {
        self.message
    }
}
