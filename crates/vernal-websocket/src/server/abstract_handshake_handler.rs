//! 对应 Java 类：org.springframework.web.socket.server.support.AbstractHandshakeHandler
//!
//! 模板方法：校验请求、子协议协商、扩展协商，最终委托 `RequestUpgradeStrategy`。
//! 对标 Spring `AbstractHandshakeHandler`。

use std::sync::Arc;

use crate::{
    HandshakeRequest, WebSocketError, WebSocketExtension, WebSocketHandler, negotiate_subprotocol,
    server::{HandshakeHandler, RequestUpgradeStrategy},
};

/// 默认握手 handler。
pub struct AbstractHandshakeHandler {
    upgrade_strategy: Arc<dyn RequestUpgradeStrategy>,
    supported_protocols: Vec<String>,
}

impl AbstractHandshakeHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(upgrade_strategy: Arc<dyn RequestUpgradeStrategy>) -> Self {
        Self {
            upgrade_strategy,
            supported_protocols: Vec::new(),
        }
    }

    /// 设置支持的子协议（按 Spring 行为小写存储）。
    #[must_use]
    pub fn with_supported_protocols(mut self, protocols: impl IntoIterator<Item = String>) -> Self {
        self.supported_protocols = protocols
            .into_iter()
            .map(|p| p.to_ascii_lowercase())
            .collect();
        self
    }

    /// 返回支持的子协议。
    #[must_use]
    pub fn supported_protocols(&self) -> &[String] {
        &self.supported_protocols
    }

    /// 返回升级策略。
    #[must_use]
    pub fn upgrade_strategy(&self) -> &Arc<dyn RequestUpgradeStrategy> {
        &self.upgrade_strategy
    }

    /// 校验握手请求是否符合 RFC 6455 基本要求。
    pub fn validate(&self, request: &HandshakeRequest) -> Result<(), WebSocketError> {
        request.validate()
    }

    /// 协商子协议。
    #[must_use]
    pub fn select_protocol(&self, request: &HandshakeRequest) -> Option<String> {
        let header_value = request
            .headers
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())?;
        let requested: Vec<&str> = header_value.split(',').map(str::trim).collect();
        negotiate_subprotocol(&requested, &self.supported_protocols)
    }

    /// 协商扩展（保留请求顺序；与 strategy 支持的扩展按 name 取交集）。
    #[must_use]
    pub fn select_extensions(&self, request: &HandshakeRequest) -> Vec<WebSocketExtension> {
        let Some(header_value) = request
            .headers
            .get("sec-websocket-extensions")
            .and_then(|value: &http::HeaderValue| value.to_str().ok())
        else {
            return Vec::new();
        };
        let Ok(requested) = WebSocketExtension::parse_extensions(header_value) else {
            return Vec::new();
        };
        let supported = self.upgrade_strategy.supported_extensions();
        let supported_names: Vec<&str> = supported.iter().map(WebSocketExtension::name).collect();
        requested
            .into_iter()
            .filter(|ext| supported_names.contains(&ext.name()))
            .collect()
    }
}

impl HandshakeHandler for AbstractHandshakeHandler {
    fn do_handshake(
        &self,
        request: HandshakeRequest,
        handler: Arc<dyn WebSocketHandler>,
        _attributes: std::collections::BTreeMap<String, String>,
    ) -> crate::server::HandshakeFuture<'_> {
        Box::pin(async move {
            self.validate(&request)?;
            let selected_protocol = self.select_protocol(&request);
            let selected_extensions = self.select_extensions(&request);
            self.upgrade_strategy.upgrade(
                selected_protocol.as_deref(),
                selected_extensions,
                handler,
            )
        })
    }
}
