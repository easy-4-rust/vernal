//! 对应 Java 类：org.springframework.web.socket.handler.SessionLimitExceededException
//!
//! 当 WebSocket session 超过配置限制（发送时长、buffer 大小等）时抛出。

use std::fmt;

use crate::{CloseCode, CloseStatus, CloseStatusError, WebSocketError};

/// Session 超限错误。等价 Spring `SessionLimitExceededException`。
#[derive(Debug)]
pub struct SessionLimitExceededError {
    /// 错误消息。
    pub message: String,
    /// 关闭状态。
    pub status: CloseStatus,
}

impl SessionLimitExceededError {
    /// 创建错误。
    pub fn new(message: impl Into<String>, status: Option<CloseStatus>) -> Self {
        Self {
            message: message.into(),
            status: status.unwrap_or_else(|| {
                CloseStatus::new(CloseCode::Custom(4500), "Session not reliable")
                    .unwrap_or_else(|_| CloseStatus::normal())
            }),
        }
    }
}

impl fmt::Display for SessionLimitExceededError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for SessionLimitExceededError {}

impl From<SessionLimitExceededError> for crate::WebSocketError {
    fn from(_error: SessionLimitExceededError) -> Self {
        // Spring 把 status 附加在异常上；调用方可在收到错误后用 `error.status` 关闭连接。
        WebSocketError::LimitExceeded {
            kind: "session",
            limit: 0,
            observed: 0,
        }
    }
}

/// 关闭状态校验时的辅助构造（不允许的 status 退化为 `NO_STATUS_CODE`）。
pub(crate) fn safe_status(code: CloseCode, reason: &str) -> CloseStatus {
    match CloseStatus::new(code, reason) {
        Ok(status) => status,
        Err(CloseStatusError::InvalidCode(_)) => CloseStatus::normal(),
        Err(CloseStatusError::ReasonTooLong(_)) => CloseStatus::normal(),
    }
}
