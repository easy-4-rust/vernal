//! JavaMail 适配 — 对标 `org.springframework.mail.javamail`。
//!
//! 使用 `lettre` crate 作为 Rust 端的邮件发送实现。

mod configurable_mime_file_type_map;
mod internet_address_editor;
mod java_mail_sender;
mod mime_mail_message;
mod mime_message;
mod mime_message_helper;
mod mime_message_preparator;
mod smart_mime_message;

pub use configurable_mime_file_type_map::ConfigurableMimeFileTypeMap;
pub use internet_address_editor::InternetAddressEditor;
pub use java_mail_sender::{JavaMailSender, LettreJavaMailSender};
pub use mime_mail_message::MimeMailMessage;
pub use mime_message::MimeMessage;
pub use mime_message_helper::MimeMessageHelper;
pub use mime_message_preparator::{MimeMessagePreparator, SimpleMimeMessagePreparator};
pub use smart_mime_message::SmartMimeMessage;
