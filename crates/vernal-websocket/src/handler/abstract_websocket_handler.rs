//! 对应 Java 类：org.springframework.web.socket.handler.AbstractWebSocketHandler
//!
//! 便利基类，按消息类型（Text/Binary/Pong）分派到 `handle_text_message` /
//! `handle_binary_message` / `handle_pong_message`，其余消息抛出协议错误。
//! 对标 Spring `AbstractWebSocketHandler`。

use std::{future::Future, pin::Pin};

use crate::{
    CloseStatus, HandlerFuture, MessageKind, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession,
};

/// 抽象 WebSocket Handler，按消息类型分派。
///
/// 等价 Spring 的 `AbstractWebSocketHandler`：默认空实现，并把
/// `on_message` 拆解为文本/二进制/Pong 三个钩子。
pub struct AbstractWebSocketHandler;

fn boxed<'a, T: Send + 'a>(
    future: impl Future<Output = T> + Send + 'a,
) -> Pin<Box<dyn Future<Output = T> + Send + 'a>> {
    Box::pin(future)
}

impl WebSocketHandler for AbstractWebSocketHandler {
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            match message.kind() {
                MessageKind::Text => {
                    let reason = WebSocketMessage::text_into(message)?;
                    self.handle_text_message(session, reason).await
                }
                MessageKind::Binary => {
                    let payload = WebSocketMessage::binary_into(message)?;
                    self.handle_binary_message(session, payload).await
                }
                MessageKind::Pong => {
                    let payload = WebSocketMessage::pong_into(message)?;
                    self.handle_pong_message(session, payload).await
                }
                MessageKind::Ping => Ok(()),
                MessageKind::Close => Ok(()),
                MessageKind::Continuation => Err(WebSocketError::protocol(
                    crate::CloseCode::ProtocolError,
                    "意外的分片消息",
                )),
            }
        })
    }
}

/// 按消息类型分派的回调 trait。
pub trait MessageHandlerHooks: Send + Sync {
    /// 处理文本消息。
    fn handle_text_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _text: String,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }

    /// 处理二进制消息。
    fn handle_binary_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _payload: bytes::Bytes,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }

    /// 处理 Pong 控制消息。
    fn handle_pong_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _payload: bytes::Bytes,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }
}

impl AbstractWebSocketHandler {
    /// 处理文本消息的默认空实现。
    pub fn handle_text_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _text: String,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }

    /// 处理二进制消息的默认空实现。
    pub fn handle_binary_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _payload: bytes::Bytes,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }

    /// 处理 Pong 消息的默认空实现。
    pub fn handle_pong_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _payload: bytes::Bytes,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        boxed(async { Ok(()) })
    }
}

impl WebSocketMessage {
    /// 将文本消息拆解为 `String`，否则返回协议错误。
    pub fn text_into(self) -> Result<String, WebSocketError> {
        match self {
            Self::Text(text) => Ok(text),
            other => Err(WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                format!("期望文本消息，实际为 {:?}", other.kind()),
            )),
        }
    }

    /// 将二进制消息拆解为 `Bytes`，否则返回协议错误。
    pub fn binary_into(self) -> Result<bytes::Bytes, WebSocketError> {
        match self {
            Self::Binary(payload) => Ok(payload),
            other => Err(WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                format!("期望二进制消息，实际为 {:?}", other.kind()),
            )),
        }
    }

    /// 将 Pong 消息拆解为 `Bytes`，否则返回协议错误。
    pub fn pong_into(self) -> Result<bytes::Bytes, WebSocketError> {
        match self {
            Self::Pong(payload) => Ok(payload),
            other => Err(WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                format!("期望 Pong 消息，实际为 {:?}", other.kind()),
            )),
        }
    }
}

/// 用于把 `CloseStatus::NOT_ACCEPTABLE` 作为单点来源。
pub const NOT_ACCEPTABLE_REASON_TEXT: &str = "Text messages not supported";
/// 用于把 `CloseStatus::NOT_ACCEPTABLE` 作为单点来源。
pub const NOT_ACCEPTABLE_REASON_BINARY: &str = "Binary messages not supported";

/// 创建 Spring `CloseStatus.NOT_ACCEPTABLE` 等价关闭状态。
#[must_use]
pub fn not_acceptable(reason: &'static str) -> CloseStatus {
    CloseStatus::new(crate::CloseCode::PolicyViolation, reason)
        .unwrap_or_else(|_| CloseStatus::normal())
}
