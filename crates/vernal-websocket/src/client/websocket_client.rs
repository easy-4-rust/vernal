//! 对应 Java 类：org.springframework.web.socket.client.WebSocketClient
//!
//! 客户端 SPI：发起握手并驱动 handler。Spring 用 `CompletableFuture<WebSocketSession>`，
//! Vernal 用 `Future<WebSocketSession>`。

use std::{future::Future, pin::Pin, sync::Arc};

use http::Uri;

use crate::{WebSocketError, WebSocketHandler, WebSocketSession};

/// 客户端连接返回的 future。
pub type ConnectFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Arc<dyn WebSocketSession>, WebSocketError>> + Send + 'a>>;

/// 客户端连接 trait。
pub trait WebSocketClient: Send + Sync {
    /// 发起握手连接，返回建立的 session。
    fn execute<'a>(
        &'a self,
        handler: Arc<dyn WebSocketHandler>,
        uri: &'a Uri,
        headers: Option<&'a http::HeaderMap>,
    ) -> ConnectFuture<'a>;
}
