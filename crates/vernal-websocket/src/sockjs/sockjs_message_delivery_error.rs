//! 对应 Java 类：org.springframework.web.socket.sockjs.SockJsMessageDeliveryException
//!
//! 消息帧已通过 HTTP POST 接收并解析，但其中一条或多条消息未交付给 handler。
//! 此异常不会自动关闭 session。

use crate::sockjs::sockjs_error::SockJsError;

/// 消息交付失败异常。
#[derive(Debug)]
pub struct SockJsMessageDeliveryError {
    /// 基础 SockJS 错误。
    pub inner: SockJsError,
    /// 未交付的消息列表。
    pub undelivered_messages: Vec<String>,
}

impl SockJsMessageDeliveryError {
    /// 创建错误（来自上层 cause）。
    #[must_use]
    pub fn from_cause(
        session_id: impl Into<String>,
        undelivered: Vec<String>,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        let session_id = session_id.into();
        let message =
            format!("Failed to deliver message(s) {undelivered:?} for session {session_id}");
        Self {
            inner: SockJsError::new(message, Some(session_id), cause),
            undelivered_messages: undelivered,
        }
    }

    /// 创建错误（来自显式消息）。
    #[must_use]
    pub fn from_message(
        session_id: impl Into<String>,
        undelivered: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        let session_id = session_id.into();
        let reason = reason.into();
        let message = format!(
            "Failed to deliver message(s) {undelivered:?} for session {session_id}: {reason}"
        );
        Self {
            inner: SockJsError::new(message, Some(session_id), None),
            undelivered_messages: undelivered,
        }
    }
}

impl std::fmt::Display for SockJsMessageDeliveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.inner)
    }
}

impl std::error::Error for SockJsMessageDeliveryError {}
