//! 对应 Java 类：org.springframework.web.socket.handler.ExceptionWebSocketHandlerDecorator
//!
//! 捕获被装饰 handler 抛出的错误；一旦发生异常，使用 `CloseStatus.SERVER_ERROR`
//! 关闭 session（Spring 行为）。

use std::sync::Arc;

use crate::{
    CloseCode, CloseStatus, HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage,
    WebSocketSession,
};

/// 异常处理装饰器。
pub struct ExceptionWebSocketHandlerDecorator {
    inner: crate::handler::WebSocketHandlerDecorator,
}

impl ExceptionWebSocketHandlerDecorator {
    /// 创建装饰器。
    #[must_use]
    pub fn new(delegate: Arc<dyn WebSocketHandler>) -> Self {
        Self {
            inner: crate::handler::WebSocketHandlerDecorator::new(delegate),
        }
    }

    /// 等价 Spring `tryCloseWithError`：若 session 仍打开，则关闭为 `SERVER_ERROR`。
    pub async fn try_close_with_error(session: &dyn WebSocketSession, error: &WebSocketError) {
        // Spring 行为：记录日志后关闭为 SERVER_ERROR（1011）
        let _ = error;
        if session.state() == crate::SessionState::Open {
            let status = CloseStatus::new(CloseCode::ServerError, "Unhandled exception")
                .unwrap_or_else(|_| CloseStatus::normal());
            let _ = session.close(status).await;
        }
    }
}

impl WebSocketHandler for ExceptionWebSocketHandlerDecorator {
    fn on_open<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if let Err(error) = self.inner.on_open(session).await {
                Self::try_close_with_error(session, &error).await;
                return Err(error);
            }
            Ok(())
        })
    }
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if let Err(error) = self.inner.on_message(session, message).await {
                Self::try_close_with_error(session, &error).await;
                return Err(error);
            }
            Ok(())
        })
    }
    fn on_error<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        error: &'a WebSocketError,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            self.inner.on_error(session, error).await;
        })
    }
    fn on_close<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        status: CloseStatus,
    ) -> HandlerFuture<'a> {
        Box::pin(async move {
            let () = self.inner.on_close(session, status).await;
        })
    }
    fn supports_partial_messages(&self) -> bool {
        self.inner.supports_partial_messages()
    }
}
