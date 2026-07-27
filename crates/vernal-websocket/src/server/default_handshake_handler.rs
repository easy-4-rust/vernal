//! 对应 Java 类：org.springframework.web.socket.server.support.DefaultHandshakeHandler
//!
//! Spring 默认使用 `StandardWebSocketUpgradeStrategy`。Vernal 中用户必须显式
//! 提供 `RequestUpgradeStrategy`（如 tokio-websockets 适配），因此本类型是
//! `AbstractHandshakeHandler` 的轻量包装。

use std::sync::Arc;

use crate::server::{AbstractHandshakeHandler, RequestUpgradeStrategy};

/// 默认握手 handler。
pub struct DefaultHandshakeHandler {
    inner: AbstractHandshakeHandler,
}

impl DefaultHandshakeHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(upgrade_strategy: Arc<dyn RequestUpgradeStrategy>) -> Self {
        Self {
            inner: AbstractHandshakeHandler::new(upgrade_strategy),
        }
    }

    /// 设置支持的子协议。
    #[must_use]
    pub fn with_supported_protocols(mut self, protocols: impl IntoIterator<Item = String>) -> Self {
        self.inner = self.inner.with_supported_protocols(protocols);
        self
    }
}

impl crate::server::HandshakeHandler for DefaultHandshakeHandler {
    fn do_handshake(
        &self,
        request: crate::HandshakeRequest,
        handler: Arc<dyn crate::WebSocketHandler>,
        attributes: std::collections::BTreeMap<String, String>,
    ) -> crate::server::HandshakeFuture<'_> {
        self.inner.do_handshake(request, handler, attributes)
    }
}
