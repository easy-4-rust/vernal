//! 对应 Java 类：org.springframework.web.socket.handler.BinaryWebSocketHandler
//!
//! 仅处理二进制消息；收到文本消息时以 `CloseStatus.NOT_ACCEPTABLE` 关闭连接。

use crate::{
    HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage, WebSocketSession,
    handler::abstract_websocket_handler::{NOT_ACCEPTABLE_REASON_TEXT, not_acceptable},
};

/// 二进制 WebSocket Handler。
#[derive(Default)]
pub struct BinaryWebSocketHandler;

impl BinaryWebSocketHandler {
    /// 创建默认 handler。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl WebSocketHandler for BinaryWebSocketHandler {
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if matches!(message.kind(), crate::MessageKind::Text) {
                session
                    .close(not_acceptable(NOT_ACCEPTABLE_REASON_TEXT))
                    .await
            } else {
                Ok(())
            }
        })
    }
}
