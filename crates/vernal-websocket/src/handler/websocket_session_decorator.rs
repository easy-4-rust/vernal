//! 对应 Java 类：org.springframework.web.socket.handler.WebSocketSessionDecorator
//!
//! 包装另一个 `WebSocketSession`，按 Spring 行为委托所有方法。
//!
//! Rust 中没有类型层级的 `instanceof`，因此 `get_last_session` / `unwrap` 由
//! 装饰器内部 `delegate` 引用直接返回（Rust 借鉴 Spring 的 unwrap 语义）。

use std::sync::Arc;

use crate::{
    CloseStatus, HandlerFuture, SessionState, WebSocketError, WebSocketMessage, WebSocketSession,
};

/// session 装饰器。
#[derive(Clone)]
pub struct WebSocketSessionDecorator {
    delegate: Arc<dyn WebSocketSession>,
}

impl WebSocketSessionDecorator {
    /// 创建装饰器。
    #[must_use]
    pub fn new(delegate: Arc<dyn WebSocketSession>) -> Self {
        Self { delegate }
    }

    /// 返回被装饰 session。
    #[must_use]
    pub fn delegate(&self) -> &Arc<dyn WebSocketSession> {
        &self.delegate
    }

    /// 返回装饰链最末端 session。
    #[must_use]
    pub fn last_session(&self) -> &Arc<dyn WebSocketSession> {
        &self.delegate
    }
}

impl WebSocketSession for WebSocketSessionDecorator {
    fn id(&self) -> &str {
        self.delegate.id()
    }
    fn uri(&self) -> Option<&http::Uri> {
        self.delegate.uri()
    }
    fn headers(&self) -> &http::HeaderMap {
        self.delegate.headers()
    }
    fn state(&self) -> SessionState {
        self.delegate.state()
    }
    fn accepted_protocol(&self) -> Option<&str> {
        self.delegate.accepted_protocol()
    }
    fn send(&self, message: WebSocketMessage) -> HandlerFuture<'_, Result<(), WebSocketError>> {
        self.delegate.send(message)
    }
    fn close(&self, status: CloseStatus) -> HandlerFuture<'_, Result<(), WebSocketError>> {
        self.delegate.close(status)
    }
}

impl std::fmt::Debug for WebSocketSessionDecorator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WebSocketSessionDecorator")
            .finish_non_exhaustive()
    }
}
