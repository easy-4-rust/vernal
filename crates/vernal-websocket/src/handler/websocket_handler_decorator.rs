//! 对应 Java 类：org.springframework.web.socket.handler.WebSocketHandlerDecorator
//!
//! 包装另一个 `WebSocketHandler`，按 Spring 装饰器链语义委托。

use std::sync::Arc;

use crate::{
    CloseStatus, HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession,
};

/// WebSocket Handler 装饰器基类。
pub struct WebSocketHandlerDecorator {
    delegate: Arc<dyn WebSocketHandler>,
}

impl WebSocketHandlerDecorator {
    /// 创建装饰器。
    #[must_use]
    pub fn new(delegate: Arc<dyn WebSocketHandler>) -> Self {
        Self { delegate }
    }

    /// 返回直接被装饰的 handler。
    #[must_use]
    pub fn delegate(&self) -> &Arc<dyn WebSocketHandler> {
        &self.delegate
    }

    /// 静态 unwrap 工具：返回装饰器链最末端的 handler。
    ///
    /// Rust 中装饰器以组合而非类型层级表达，因此等价于返回 `delegate`。
    #[must_use]
    pub fn last_handler(&self) -> &Arc<dyn WebSocketHandler> {
        &self.delegate
    }
}

impl WebSocketHandler for WebSocketHandlerDecorator {
    fn on_open<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        self.delegate.on_open(session)
    }
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        self.delegate.on_message(session, message)
    }
    fn on_error<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        self.delegate.on_error(session, error)
    }
    fn on_close<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        status: CloseStatus,
    ) -> HandlerFuture<'a> {
        self.delegate.on_close(session, status)
    }
    fn supports_partial_messages(&self) -> bool {
        self.delegate.supports_partial_messages()
    }
}
