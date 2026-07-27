//! 对应 Java 类：org.springframework.web.socket.server.HandshakeFailureException
//!
//! 握手处理因内部不可恢复错误失败时抛出，对应 HTTP 500，区别于握手协商失败。

use std::fmt;

use crate::WebSocketError;

/// 握手失败错误。
#[derive(Debug)]
pub struct HandshakeFailureError {
    /// 失败原因。
    pub reason: String,
}

impl HandshakeFailureError {
    /// 创建失败错误。
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl fmt::Display for HandshakeFailureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for HandshakeFailureError {}

impl From<HandshakeFailureError> for WebSocketError {
    fn from(error: HandshakeFailureError) -> Self {
        WebSocketError::Handshake {
            status: 500,
            reason: error.reason,
        }
    }
}
