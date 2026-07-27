//! WebSocket 传输适配。

use std::time::Duration;

use crate::{WebSocketError, WebSocketMessage};

/// 框架无关的 WebSocket 传输能力。
pub trait WebSocketTransport: Send + Sync {
    /// 返回传输名称。
    fn name(&self) -> &'static str;
    /// 返回最大 payload 限制。
    fn max_payload_size(&self) -> Option<usize>;
}

/// tokio-websockets 传输配置描述。
#[derive(Debug, Clone)]
pub struct TokioWebsocketsTransport {
    max_payload_size: Option<usize>,
    enqueue_timeout: Duration,
}

impl TokioWebsocketsTransport {
    /// 创建默认 tokio-websockets transport 描述。
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_payload_size: Some(1024 * 1024),
            enqueue_timeout: Duration::from_secs(5),
        }
    }

    /// 设置最大 payload 大小。
    #[must_use]
    pub const fn max_payload_size(mut self, limit: Option<usize>) -> Self {
        self.max_payload_size = limit;
        self
    }

    /// 设置发送超时。
    #[must_use]
    pub const fn enqueue_timeout(mut self, timeout: Duration) -> Self {
        self.enqueue_timeout = timeout;
        self
    }

    /// 返回发送超时。
    #[must_use]
    pub const fn send_timeout(&self) -> Duration {
        self.enqueue_timeout
    }
}

impl Default for TokioWebsocketsTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketTransport for TokioWebsocketsTransport {
    fn name(&self) -> &'static str {
        "tokio-websockets"
    }
    fn max_payload_size(&self) -> Option<usize> {
        self.max_payload_size
    }
}

/// 将公共消息转换为 tokio-websockets 消息。
#[cfg(feature = "tokio-websockets")]
pub fn to_tokio_message(
    message: WebSocketMessage,
) -> Result<tokio_websockets::Message, WebSocketError> {
    use tokio_websockets::Message;
    match message {
        WebSocketMessage::Text(text) => Ok(Message::text(text)),
        WebSocketMessage::Binary(payload) => Ok(Message::binary(payload)),
        WebSocketMessage::Ping(payload) => Ok(Message::ping(payload)),
        WebSocketMessage::Pong(payload) => Ok(Message::pong(payload)),
        WebSocketMessage::Close(status) => {
            let (code, reason) = if let Some(status) = status {
                let code = tokio_websockets::CloseCode::try_from(status.code().as_u16()).map_err(
                    |_| WebSocketError::protocol(crate::CloseCode::ProtocolError, "无效关闭码"),
                )?;
                (Some(code), status.reason().to_owned())
            } else {
                (None, String::new())
            };
            Ok(Message::close(code, &reason))
        }
        WebSocketMessage::Continuation { payload, .. } => Ok(Message::binary(payload)),
    }
}

/// 将 tokio-websockets 消息转换为公共消息。
#[cfg(feature = "tokio-websockets")]
pub fn from_tokio_message(
    message: tokio_websockets::Message,
) -> Result<WebSocketMessage, WebSocketError> {
    if let Some(text) = message.as_text() {
        return Ok(WebSocketMessage::text(text));
    }
    if message.is_binary() {
        return Ok(WebSocketMessage::binary(message.into_payload()));
    }
    if message.is_ping() {
        return Ok(WebSocketMessage::Ping(message.into_payload().into()));
    }
    if message.is_pong() {
        return Ok(WebSocketMessage::Pong(message.into_payload().into()));
    }
    if let Some((code, reason)) = message.as_close() {
        let code = match u16::from(code) {
            1000 => crate::CloseCode::Normal,
            1001 => crate::CloseCode::GoingAway,
            1002 => crate::CloseCode::ProtocolError,
            1003 => crate::CloseCode::UnsupportedData,
            1007 => crate::CloseCode::InvalidPayload,
            1008 => crate::CloseCode::PolicyViolation,
            1009 => crate::CloseCode::MessageTooBig,
            1010 => crate::CloseCode::MandatoryExtension,
            1011 => crate::CloseCode::ServerError,
            other => crate::CloseCode::Custom(other),
        };
        return crate::CloseStatus::new(code, reason)
            .map(|status| WebSocketMessage::Close(Some(status)))
            .map_err(WebSocketError::from);
    }
    Err(WebSocketError::protocol(
        crate::CloseCode::ProtocolError,
        "不支持的 tokio-websockets 消息",
    ))
}
