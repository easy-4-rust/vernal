//! 邮件支持模块 — 对标 `org.springframework.mail`。
//!
//! 包含：
//! - `MailSender` trait：邮件发送器
//! - `MailMessage` trait：邮件消息
//! - `SimpleMailMessage`：简单邮件消息
//! - `javamail`：JavaMail 适配（lettre 后端）

mod mail_exception;
mod mail_message;
mod mail_sender;
mod simple_mail_message;

#[cfg(feature = "mail")]
pub mod javamail;

pub use mail_exception::*;
pub use mail_message::MailMessage;
pub use mail_sender::MailSender;
pub use simple_mail_message::SimpleMailMessage;
