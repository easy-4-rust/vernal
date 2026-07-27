//! 对应 Java 类：org.springframework.web.socket.server.RequestUpgradeStrategy
//!
//! 在握手成功后由 `HandshakeHandler` 调用，把 HTTP 连接升级为 WebSocket。
//! Spring 实现（StandardWebSocketUpgradeStrategy/JettyRequestUpgradeStrategy）
//! 与 Servlet/Jetty API 强耦合；Vernal 通过 transport 抽象解耦。

use std::sync::Arc;

use crate::{WebSocketExtension, WebSocketHandler, WebSocketSession};

/// 请求升级策略 SPI。
pub trait RequestUpgradeStrategy: Send + Sync {
    /// 返回支持的 WebSocket 协议版本。
    fn supported_versions(&self) -> &'static [&'static str];

    /// 返回 transport 支持的扩展（对标 Spring `getSupportedExtensions`）。
    fn supported_extensions(&self) -> Vec<WebSocketExtension>;

    /// 完成 HTTP 到 WebSocket 的升级。
    ///
    /// 成功时返回 session。
    fn upgrade(
        &self,
        selected_protocol: Option<&str>,
        selected_extensions: Vec<WebSocketExtension>,
        handler: Arc<dyn WebSocketHandler>,
    ) -> Result<Arc<dyn WebSocketSession>, crate::WebSocketError>;
}
