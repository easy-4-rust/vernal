//! 智能 MIME 消息 — 对标 `SmartMimeMessage`。

use super::mime_message::MimeMessage;

/// 智能 MIME 消息。
///
/// 对标 Spring 的 `SmartMimeMessage`，携带默认编码和默认 FileTypeMap。
pub struct SmartMimeMessage {
    message: MimeMessage,
    default_encoding: Option<String>,
}

impl SmartMimeMessage {
    /// 创建智能 MIME 消息。
    pub fn new(message: MimeMessage, default_encoding: Option<String>) -> Self {
        Self {
            message,
            default_encoding,
        }
    }

    /// 获取默认编码。
    pub fn default_encoding(&self) -> Option<&str> {
        self.default_encoding.as_deref()
    }

    /// 获取内部 MIME 消息。
    pub fn mime_message(&self) -> &MimeMessage {
        &self.message
    }
}
