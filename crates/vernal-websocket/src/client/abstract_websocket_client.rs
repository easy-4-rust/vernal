//! 对应 Java 类：org.springframework.web.socket.client.AbstractWebSocketClient
//!
//! 客户端实现基类，对标 Spring `AbstractWebSocketClient`。
//! Rust 中以 trait default 方法 + 共享配置表达。

use std::sync::Arc;

use http::{HeaderMap, Uri};

use crate::{
    WebSocketHandler,
    client::{ConnectFuture, WebSocketClient},
};

/// 抽象客户端配置共享。
#[derive(Debug, Clone, Default)]
pub struct AbstractWebSocketClient {
    /// 默认握手 headers。
    pub default_headers: HeaderMap,
}

impl AbstractWebSocketClient {
    /// 创建空配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认 headers。
    #[must_use]
    pub fn with_default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }
}

/// 把 handler + uri + headers 委派给真实 client 的默认实现。
pub fn delegate_execute<'a>(
    client: &'a (dyn WebSocketClient + 'a),
    handler: Arc<dyn WebSocketHandler>,
    uri: &'a Uri,
    headers: Option<&'a HeaderMap>,
) -> ConnectFuture<'a> {
    client.execute(handler, uri, headers)
}
