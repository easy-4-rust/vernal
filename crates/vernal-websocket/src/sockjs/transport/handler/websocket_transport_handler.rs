//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.handler.WebSocketTransportHandler
//! 与 SockJsWebSocketHandler
//!
//! WebSocket 传输 handler：把 SockJS message 帧通过 WebSocket session 发送/接收。
//! 对标 Spring `WebSocketTransportHandler` + `SockJsWebSocketHandler`。

use std::sync::Arc;

use crate::sockjs::transport::TransportType;
use crate::sockjs::transport::handler::abstract_transport_handler::AbstractTransportHandler;
use crate::sockjs::transport::session::abstract_http_sockjs_session::WebSocketServerSockJsSession;
use crate::sockjs::transport::transport_handler::{TransportHandleFuture, TransportHandler};

/// WebSocket transport handler。
pub struct WebSocketTransportHandler {
    base: AbstractTransportHandler,
}

impl WebSocketTransportHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: AbstractTransportHandler::new(),
        }
    }
}

impl Default for WebSocketTransportHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportHandler for WebSocketTransportHandler {
    fn initialize(&self, config: crate::sockjs::transport::SockJsServiceConfig) {
        self.base.set_config(config);
    }
    fn transport_type(&self) -> TransportType {
        TransportType::WebSocket
    }
    fn check_session_type(
        &self,
        _session: &dyn crate::sockjs::transport::sockjs_session::SockJsSession,
    ) -> bool {
        true
    }
    fn handle_request(
        &self,
        _handler: Arc<dyn crate::WebSocketHandler>,
        _session: Arc<dyn crate::sockjs::transport::sockjs_session::SockJsSession>,
    ) -> TransportHandleFuture {
        Box::pin(async { Ok(()) })
    }
}

/// SockJsWebSocketHandler：包装 WebSocket session 并把 SockJS 帧路由到底层。
pub struct SockJsWebSocketHandler {
    sockjs_session: Arc<WebSocketServerSockJsSession>,
}

impl SockJsWebSocketHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(sockjs_session: Arc<WebSocketServerSockJsSession>) -> Self {
        Self { sockjs_session }
    }

    /// 返回关联的 SockJS session。
    #[must_use]
    pub fn sockjs_session(&self) -> &Arc<WebSocketServerSockJsSession> {
        &self.sockjs_session
    }
}

impl crate::WebSocketHandler for SockJsWebSocketHandler {
    fn on_open<'a>(
        &'a self,
        _session: &'a dyn crate::WebSocketSession,
    ) -> crate::HandlerFuture<'a, Result<(), crate::WebSocketError>> {
        Box::pin(async move {
            self.sockjs_session.base().open().await;
            Ok(())
        })
    }

    fn on_message<'a>(
        &'a self,
        _session: &'a dyn crate::WebSocketSession,
        message: crate::WebSocketMessage,
    ) -> crate::HandlerFuture<'a, Result<(), crate::WebSocketError>> {
        Box::pin(async move {
            if let crate::WebSocketMessage::Text(text) = message {
                self.sockjs_session.base().enqueue_message(text).await;
            }
            Ok(())
        })
    }

    fn on_close<'a>(
        &'a self,
        _session: &'a dyn crate::WebSocketSession,
        _status: crate::CloseStatus,
    ) -> crate::HandlerFuture<'a> {
        Box::pin(async move {
            self.sockjs_session.base().close().await;
        })
    }
}
