//! 对应 Java 类：org.springframework.web.socket.server.HandshakeHandler
//!
//! 处理 WebSocket 握手请求的 SPI。

use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use crate::{WebSocketError, WebSocketHandler, WebSocketSession};

/// 握手 handler 返回的 future。
pub type HandshakeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Arc<dyn WebSocketSession>, WebSocketError>> + Send + 'a>>;

/// 握手 handler trait。
pub trait HandshakeHandler: Send + Sync {
    /// 发起握手。成功时返回创建好的 `WebSocketSession`。
    fn do_handshake(
        &self,
        request: crate::HandshakeRequest,
        handler: Arc<dyn WebSocketHandler>,
        attributes: BTreeMap<String, String>,
    ) -> HandshakeFuture<'_>;
}
