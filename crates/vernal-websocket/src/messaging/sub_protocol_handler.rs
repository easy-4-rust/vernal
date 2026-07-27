//! 对应 Java 类：org.springframework.web.socket.messaging.SubProtocolHandler
//!
//! 子协议处理 SPI。对标 Spring `SubProtocolHandler`，使用 Rust 异步 Future。

use std::{future::Future, pin::Pin, sync::Arc};

use vernal_messaging::{Message, MessageChannel};

use crate::{CloseStatus, WebSocketError, WebSocketMessage, WebSocketSession};

/// 子协议 handler 返回的 future。
pub type SubProtocolFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), WebSocketError>> + Send + 'a>>;

/// 子协议处理 SPI。
pub trait SubProtocolHandler: Send + Sync {
    /// 返回支持的子协议列表（按 Spring 行为小写）。
    fn supported_protocols(&self) -> Vec<String>;

    /// 处理来自客户端的 WebSocket 消息。
    fn handle_message_from_client<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
        output_channel: Arc<dyn MessageChannel>,
    ) -> SubProtocolFuture<'a>;

    /// 处理要发送给客户端的 `Message`。
    fn handle_message_to_client<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: Arc<dyn Message>,
    ) -> SubProtocolFuture<'a>;

    /// 从消息中解析 session id。
    fn resolve_session_id(&self, message: &dyn Message) -> Option<String>;

    /// session 启动后回调。
    fn after_session_started<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        output_channel: Arc<dyn MessageChannel>,
    ) -> SubProtocolFuture<'a> {
        let _ = session;
        let _ = output_channel;
        Box::pin(async { Ok(()) })
    }

    /// session 结束后回调。
    fn after_session_ended<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        close_status: CloseStatus,
        output_channel: Arc<dyn MessageChannel>,
    ) -> SubProtocolFuture<'a> {
        let _ = (session, close_status, output_channel);
        Box::pin(async { Ok(()) })
    }
}
