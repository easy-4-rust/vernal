//! MIME 消息 — 对标 `jakarta.mail.internet.MimeMessage`。

/// MIME 消息。
///
/// 对标 Spring 的 `jakarta.mail.internet.MimeMessage`，
/// 在 Rust 中使用 `lettre::Message` 作为底层实现。
#[derive(Debug, Clone)]
pub struct MimeMessage {
    from: Option<String>,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: Option<String>,
    text_body: Option<String>,
    html_body: Option<String>,
}

impl MimeMessage {
    /// 创建空的 MIME 消息。
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: None,
            text_body: None,
            html_body: None,
        }
    }

    /// 设置发件人。
    pub fn set_from(&mut self, from: impl Into<String>) {
        self.from = Some(from.into());
    }

    /// 添加收件人。
    pub fn add_recipient(&mut self, to: impl Into<String>) {
        self.to.push(to.into());
    }

    /// 添加抄送。
    pub fn add_cc(&mut self, cc: impl Into<String>) {
        self.cc.push(cc.into());
    }

    /// 添加密送。
    pub fn add_bcc(&mut self, bcc: impl Into<String>) {
        self.bcc.push(bcc.into());
    }

    /// 设置主题。
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.subject = Some(subject.into());
    }

    /// 设置文本内容。
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text_body = Some(text.into());
    }

    /// 设置 HTML 内容。
    pub fn set_html(&mut self, html: impl Into<String>) {
        self.html_body = Some(html.into());
    }

    /// 获取发件人。
    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    /// 获取收件人列表。
    pub fn to(&self) -> &[String] {
        &self.to
    }

    /// 获取主题。
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    /// 获取文本内容。
    pub fn text_body(&self) -> Option<&str> {
        self.text_body.as_deref()
    }

    /// 获取 HTML 内容。
    pub fn html_body(&self) -> Option<&str> {
        self.html_body.as_deref()
    }

    /// 转换为 lettre 消息。
    pub fn to_lettre_message(&self) -> Result<lettre::Message, String> {
        let from = self.from.as_deref().unwrap_or("noreply@example.com");
        let to = self.to.first().ok_or("没有收件人")?;
        let subject = self.subject.as_deref().unwrap_or("");
        let body = self
            .text_body
            .as_deref()
            .or(self.html_body.as_deref())
            .unwrap_or("");

        lettre::Message::builder()
            .from(
                format!("Vernal <{from}>")
                    .parse()
                    .map_err(|e| format!("发件人地址无效：{e}"))?,
            )
            .to(to.parse().map_err(|e| format!("收件人地址无效：{e}"))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| e.to_string())
    }
}

impl Default for MimeMessage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_message() {
        let mut msg = MimeMessage::new();
        msg.set_from("sender@example.com");
        msg.add_recipient("recipient@example.com");
        msg.set_subject("Test");
        msg.set_text("Hello");

        assert_eq!(msg.from(), Some("sender@example.com"));
        assert_eq!(msg.to(), &["recipient@example.com"]);
        assert_eq!(msg.subject(), Some("Test"));
        assert_eq!(msg.text_body(), Some("Hello"));
    }
}
