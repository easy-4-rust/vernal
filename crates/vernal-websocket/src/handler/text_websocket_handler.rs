//! 对应 Java 类：org.springframework.web.socket.handler.TextWebSocketHandler
//!
//! 仅处理文本消息；收到二进制消息时以 `CloseStatus.NOT_ACCEPTABLE` 关闭连接。

use crate::{
    HandlerFuture, WebSocketError, WebSocketHandler, WebSocketMessage, WebSocketSession,
    handler::abstract_websocket_handler::{NOT_ACCEPTABLE_REASON_BINARY, not_acceptable},
};

/// 文本 WebSocket Handler。
#[derive(Default)]
pub struct TextWebSocketHandler;

impl TextWebSocketHandler {
    /// 创建默认 handler。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl WebSocketHandler for TextWebSocketHandler {
    fn on_message<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if matches!(message.kind(), crate::MessageKind::Binary) {
                session
                    .close(not_acceptable(NOT_ACCEPTABLE_REASON_BINARY))
                    .await
            } else {
                // 其余消息按基类语义接受；文本由使用者在外层继承处理。
                Ok(())
            }
        })
    }
}
