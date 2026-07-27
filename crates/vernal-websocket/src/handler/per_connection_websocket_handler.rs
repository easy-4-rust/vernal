//! 对应 Java 类：org.springframework.web.socket.handler.PerConnectionWebSocketHandler
//!
//! 每条 WebSocket 连接创建独立的 handler 实例。Spring 用 `BeanFactory` 创建/销毁
//! 目标 handler；Rust 用 `BeanCreatingHandlerProvider` 工厂等价表达。

use std::sync::{Arc, Mutex};

use tokio::sync::Mutex as AsyncMutex;

use crate::{
    CloseStatus, HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession, handler::BeanCreatingHandlerProvider,
};

/// 每连接独立 handler。
pub struct PerConnectionWebSocketHandler {
    provider: BeanCreatingHandlerProvider,
    handlers: Mutex<Vec<(String, Arc<dyn WebSocketHandler>)>>,
    supports_partial: bool,
    /// 异步保护：handlers 索引更新锁。
    handlers_lock: AsyncMutex<()>,
}

impl PerConnectionWebSocketHandler {
    /// 创建 per-connection handler。
    #[must_use]
    pub fn new(provider: BeanCreatingHandlerProvider, supports_partial_messages: bool) -> Self {
        Self {
            provider,
            handlers: Mutex::new(Vec::new()),
            supports_partial: supports_partial_messages,
            handlers_lock: AsyncMutex::new(()),
        }
    }

    async fn handler_for(
        &self,
        session: &dyn WebSocketSession,
    ) -> Result<Arc<dyn WebSocketHandler>, WebSocketError> {
        let _guard = self.handlers_lock.lock().await;
        let handlers = self.handlers.lock().expect("handlers mutex poisoned");
        handlers
            .iter()
            .find(|(id, _)| id == session.id())
            .map(|(_, handler)| Arc::clone(handler))
            .ok_or_else(|| {
                WebSocketError::protocol(
                    crate::CloseCode::ServerError,
                    format!("WebSocketHandler not found for {}", session.id()),
                )
            })
    }
}

impl WebSocketHandler for PerConnectionWebSocketHandler {
    fn on_open<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            let handler = self.provider.get_handler();
            let _guard = self.handlers_lock.lock().await;
            {
                let mut handlers = self.handlers.lock().expect("handlers mutex poisoned");
                handlers.push((session.id().to_owned(), Arc::clone(&handler)));
            }
            drop(_guard);
            handler.on_open(session).await
        })
    }

    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            let handler = self.handler_for(session).await?;
            handler.on_message(session, message).await
        })
    }

    fn on_error<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            if let Ok(handler) = self.handler_for(session).await {
                handler.on_error(session, error).await;
            }
        })
    }

    fn on_close<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        status: CloseStatus,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            let removed = {
                let _guard = self.handlers_lock.lock().await;
                let mut handlers = self.handlers.lock().expect("handlers mutex poisoned");
                let position = handlers.iter().position(|(id, _)| id == session.id());
                position.map(|index| handlers.remove(index).1)
            };
            if let Some(handler) = removed {
                handler.on_close(session, status).await;
                self.provider.destroy(&handler);
            }
        })
    }

    fn supports_partial_messages(&self) -> bool {
        self.supports_partial
    }
}

impl std::fmt::Debug for PerConnectionWebSocketHandler {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PerConnectionWebSocketHandler")
            .field("supports_partial_messages", &self.supports_partial)
            .finish_non_exhaustive()
    }
}
