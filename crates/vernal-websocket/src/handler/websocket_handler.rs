//! 对应 Java 类：org.springframework.web.socket.WebSocketHandler
//!
//! WebSocket Handler trait。对标 Spring `WebSocketHandler` 接口，
//! 使用 Rust 异步 Future 表达生命周期。

use std::{future::Future, pin::Pin};

use crate::{CloseStatus, WebSocketError, WebSocketMessage, WebSocketSession};

/// Handler 返回的动态异步任务。
pub type HandlerFuture<'a, T = ()> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// WebSocket Handler trait。
///
/// 对标 Spring 的 `WebSocketHandler`，但使用 Rust 异步 Future 表达生命周期。
pub trait WebSocketHandler: Send + Sync {
    /// 连接建立时调用。对应 `afterConnectionEstablished`。
    fn on_open<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async { Ok(()) })
    }

    /// 收到消息时调用。对应 `handleMessage`。
    fn on_message<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async { Ok(()) })
    }

    /// 发生传输错误时调用。对应 `handleTransportError`。
    fn on_error<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        Box::pin(async {})
    }

    /// 连接关闭时调用。对应 `afterConnectionClosed`。
    fn on_close<'a>(
        &'a self,
        _session: &'a dyn WebSocketSession,
        _status: CloseStatus,
    ) -> HandlerFuture<'a> {
        Box::pin(async {})
    }

    /// 返回是否接收部分消息。对应 `supportsPartialMessages`。
    #[must_use]
    fn supports_partial_messages(&self) -> bool {
        false
    }
}
