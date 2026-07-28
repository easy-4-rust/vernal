//! JavaMail 发送器 — 对标 `org.springframework.mail.javamail.JavaMailSender`。

use super::mime_message::MimeMessage;
use crate::mail::{MailError, MailMessage, MailSender};
use lettre::Transport;

/// JavaMail 发送器 trait。
///
/// 对标 Spring 的 `JavaMailSender`，支持 MIME 消息发送。
pub trait JavaMailSender: MailSender {
    /// 创建新的 MIME 消息。
    fn create_mime_message(&self) -> MimeMessage;

    /// 发送 MIME 消息。
    fn send_mime_message(&self, message: &MimeMessage) -> Result<(), MailError>;

    /// 批量发送 MIME 消息。
    fn send_mime_messages(&self, messages: &[MimeMessage]) -> Result<(), MailError>;
}

/// 基于 lettre 的 JavaMail 发送器实现。
pub struct LettreJavaMailSender {
    /// SMTP 主机
    smtp_host: String,
    /// SMTP 端口
    smtp_port: u16,
    /// 用户名
    username: Option<String>,
    /// 密码
    password: Option<String>,
}

impl LettreJavaMailSender {
    /// 创建 lettre 邮件发送器。
    pub fn new(smtp_host: String, smtp_port: u16) -> Self {
        Self {
            smtp_host,
            smtp_port,
            username: None,
            password: None,
        }
    }

    /// 设置认证信息。
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = Some(username);
        self.password = Some(password);
    }
}

impl MailSender for LettreJavaMailSender {
    fn send_message(&self, message: &dyn MailMessage) -> Result<(), MailError> {
        // 构建 lettre 消息
        let from = message.from().unwrap_or("noreply@example.com");
        let to = message
            .to()
            .first()
            .ok_or_else(|| MailError::SendError("没有收件人".to_string()))?;
        let subject = message.subject().unwrap_or("");
        let body = message.text().unwrap_or("");

        // 使用 lettre 发送
        let email = lettre::Message::builder()
            .from(
                format!("Vernal <{from}>")
                    .parse()
                    .map_err(|e| MailError::SendError(format!("发件人地址无效：{e}")))?,
            )
            .to(format!("{to}")
                .parse()
                .map_err(|e| MailError::SendError(format!("收件人地址无效：{e}")))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| MailError::SendError(format!("构建邮件失败：{e}")))?;

        // 创建传输并发送
        let transport = lettre::transport::smtp::SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| MailError::SendError(format!("连接 SMTP 服务器失败：{e}")))?
            .port(self.smtp_port);

        let transport = if let (Some(username), Some(password)) = (&self.username, &self.password) {
            transport.credentials(lettre::transport::smtp::authentication::Credentials::new(
                username.clone(),
                password.clone(),
            ))
        } else {
            transport
        };

        let transport = transport.build();
        transport
            .send(&email)
            .map_err(|e| MailError::SendError(format!("发送邮件失败：{e}")))?;

        Ok(())
    }
}

impl JavaMailSender for LettreJavaMailSender {
    fn create_mime_message(&self) -> MimeMessage {
        MimeMessage::new()
    }

    fn send_mime_message(&self, message: &MimeMessage) -> Result<(), MailError> {
        let email = message
            .to_lettre_message()
            .map_err(|e| MailError::SendError(format!("转换 MIME 消息失败：{e}")))?;

        let transport = lettre::transport::smtp::SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| MailError::SendError(format!("连接 SMTP 服务器失败：{e}")))?
            .port(self.smtp_port);

        let transport = if let (Some(username), Some(password)) = (&self.username, &self.password) {
            transport.credentials(lettre::transport::smtp::authentication::Credentials::new(
                username.clone(),
                password.clone(),
            ))
        } else {
            transport
        };

        let transport = transport.build();
        transport
            .send(&email)
            .map_err(|e| MailError::SendError(format!("发送 MIME 邮件失败：{e}")))?;

        Ok(())
    }

    fn send_mime_messages(&self, messages: &[MimeMessage]) -> Result<(), MailError> {
        for message in messages {
            self.send_mime_message(message)?;
        }
        Ok(())
    }
}
