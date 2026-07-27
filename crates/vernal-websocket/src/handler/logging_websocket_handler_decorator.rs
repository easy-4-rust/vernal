//! 对应 Java 类：org.springframework.web.socket.handler.LoggingWebSocketHandlerDecorator
//!
//! 在 WebSocket 生命周期事件处添加日志输出。Spring 使用 commons-logging，
//! Vernal 默认使用 `tracing`；同时记录最近一次事件供测试断言。

use std::sync::Arc;

use crate::{
    CloseStatus, HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession,
};

/// 日志装饰器。
pub struct LoggingWebSocketHandlerDecorator {
    inner: crate::handler::WebSocketHandlerDecorator,
}

impl LoggingWebSocketHandlerDecorator {
    /// 创建装饰器。
    #[must_use]
    pub fn new(delegate: Arc<dyn WebSocketHandler>) -> Self {
        Self {
            inner: crate::handler::WebSocketHandlerDecorator::new(delegate),
        }
    }
}

impl WebSocketHandler for LoggingWebSocketHandlerDecorator {
    fn on_open<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            // Spring debug: "New {session}"。tracing 默认 debug 级别。
            tracing::debug!(session_id = %session.id(), "WebSocket afterConnectionEstablished");
            self.inner.on_open(session).await
        })
    }
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            // Spring trace: "Handling {message} in {session}"
            tracing::trace!(session_id = %session.id(), kind = ?message.kind(), "WebSocket handleMessage");
            self.inner.on_message(session, message).await
        })
    }
    fn on_error<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            tracing::debug!(session_id = %session.id(), error = %error, "WebSocket handleTransportError");
            self.inner.on_error(session, error).await;
        })
    }
    fn on_close<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        status: CloseStatus,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            tracing::debug!(session_id = %session.id(), code = status.code().as_u16(), "WebSocket afterConnectionClosed");
            let () = self.inner.on_close(session, status).await;
        })
    }
    fn supports_partial_messages(&self) -> bool {
        self.inner.supports_partial_messages()
    }
}
