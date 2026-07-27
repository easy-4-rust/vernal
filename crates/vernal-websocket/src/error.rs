//! WebSocket 结构化错误。

use std::{error::Error, fmt};

use crate::{CloseCode, CloseStatus, CloseStatusError};

/// WebSocket 操作错误。
#[derive(Debug)]
#[non_exhaustive]
pub enum WebSocketError {
    /// HTTP 握手失败。
    Handshake {
        /// HTTP 状态码。
        status: u16,
        /// 失败原因。
        reason: String,
    },
    /// RFC 6455 或子协议错误。
    Protocol {
        /// 关闭码。
        code: CloseCode,
        /// 协议错误原因。
        reason: String,
    },
    /// 底层传输错误。
    Transport {
        /// 原始错误。
        source: Box<dyn Error + Send + Sync>,
    },
    /// 超出配置限制。
    LimitExceeded {
        /// 限制名称。
        kind: &'static str,
        /// 允许的上限。
        limit: usize,
        /// 实际观测值。
        observed: usize,
    },
    /// 连接已经关闭。
    Closed,
    /// 背压队列拒绝消息。
    Backpressure,
    /// 操作被取消。
    Cancelled,
}

impl WebSocketError {
    /// 创建传输错误。
    #[must_use]
    pub fn transport(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Transport {
            source: Box::new(error),
        }
    }

    /// 创建协议错误。
    #[must_use]
    pub fn protocol(code: CloseCode, reason: impl Into<String>) -> Self {
        Self::Protocol {
            code,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handshake { status, reason } => {
                write!(formatter, "WebSocket 握手失败 ({status}): {reason}")
            }
            Self::Protocol { code, reason } => write!(
                formatter,
                "WebSocket 协议错误 ({}): {reason}",
                code.as_u16()
            ),
            Self::Transport { source } => write!(formatter, "WebSocket 传输错误: {source}"),
            Self::LimitExceeded {
                kind,
                limit,
                observed,
            } => write!(formatter, "WebSocket {kind} 超限: {observed} > {limit}"),
            Self::Closed => formatter.write_str("WebSocket 连接已关闭"),
            Self::Backpressure => formatter.write_str("WebSocket 背压队列已满"),
            Self::Cancelled => formatter.write_str("WebSocket 操作已取消"),
        }
    }
}

impl Error for WebSocketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transport { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<CloseStatusError> for WebSocketError {
    fn from(error: CloseStatusError) -> Self {
        Self::protocol(CloseCode::ProtocolError, error.to_string())
    }
}

impl From<crate::stomp::stomp_codec::StompCodecError> for WebSocketError {
    fn from(error: crate::stomp::stomp_codec::StompCodecError) -> Self {
        Self::protocol(CloseCode::ProtocolError, error.to_string())
    }
}

impl From<WebSocketError> for CloseStatus {
    fn from(error: WebSocketError) -> Self {
        let code = match error {
            WebSocketError::Protocol { code, .. } => code,
            WebSocketError::LimitExceeded { .. } => CloseCode::MessageTooBig,
            WebSocketError::Closed => CloseCode::GoingAway,
            _ => CloseCode::ServerError,
        };
        CloseStatus::new(code, "WebSocket handler error").unwrap_or_else(|_| CloseStatus::normal())
    }
}
