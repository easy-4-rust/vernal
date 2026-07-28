//! MIME 消息准备回调 — 对标 `MimeMessagePreparator`。

use super::mime_message::MimeMessage;

/// MIME 消息准备回调 trait。
///
/// 对标 Spring 的 `MimeMessagePreparator`，用于准备 MIME 消息。
pub trait MimeMessagePreparator: Send + Sync {
    /// 准备 MIME 消息。
    fn prepare(
        &self,
        message: &mut MimeMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 简单的 MIME 消息准备回调。
pub struct SimpleMimeMessagePreparator {
    preparer: Box<
        dyn Fn(&mut MimeMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync,
    >,
}

impl SimpleMimeMessagePreparator {
    /// 创建准备回调。
    pub fn new<F>(preparer: F) -> Self
    where
        F: Fn(&mut MimeMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            preparer: Box::new(preparer),
        }
    }
}

impl MimeMessagePreparator for SimpleMimeMessagePreparator {
    fn prepare(
        &self,
        message: &mut MimeMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.preparer)(message)
    }
}
