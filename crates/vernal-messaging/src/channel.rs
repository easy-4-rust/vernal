//! 消息通道 trait。

use std::sync::Arc;

use super::message::Message;

/// 消息通道 trait。
///
/// 对标 Spring 的 `MessageChannel`。
pub trait MessageChannel: Send + Sync {
    /// 发送消息。
    fn send(&self, message: Arc<dyn Message>) -> Result<(), MessageError>;

    /// 接收消息。
    fn receive(&self) -> Result<Option<Arc<dyn Message>>, MessageError>;
}

/// 消息错误。
#[derive(Debug, Clone)]
pub struct MessageError {
    pub message: String,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "消息错误: {}", self.message)
    }
}

impl std::error::Error for MessageError {}
