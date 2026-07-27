//! 对应 Java 类：org.springframework.web.socket.server.HandshakeInterceptor
//!
//! WebSocket 握手请求拦截器。对标 Spring `HandshakeInterceptor`。

use std::{future::Future, pin::Pin, sync::Arc};

use http::HeaderMap;

use crate::{HandlerFuture, WebSocketError, WebSocketHandler};

/// 握手属性映射。
pub type HandshakeAttributes = std::collections::BTreeMap<String, String>;

/// 握手请求快照（用于拦截器视图）。
#[derive(Debug, Clone)]
pub struct HandshakeContext {
    /// 请求方法。
    pub method: http::Method,
    /// 请求 URI。
    pub uri: http::Uri,
    /// 请求头。
    pub headers: HeaderMap,
    /// 握手属性（最终注入 `WebSocketSession`）。
    pub attributes: HandshakeAttributes,
}

impl HandshakeContext {
    /// 创建握手上下文。
    #[must_use]
    pub fn new(method: http::Method, uri: http::Uri, headers: HeaderMap) -> Self {
        Self {
            method,
            uri,
            headers,
            attributes: HandshakeAttributes::new(),
        }
    }
}

/// 拦截器 before 返回的 Future。
pub type BeforeHandshakeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<bool, WebSocketError>> + Send + 'a>>;

/// 拦截器 after 返回的 Future。
pub type AfterHandshakeFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

/// WebSocket 握手拦截器 trait。
pub trait HandshakeInterceptor: Send + Sync {
    /// 握手开始前。返回 `false` 终止握手。
    fn before_handshake<'a>(
        &'a self,
        _context: &'a mut HandshakeContext,
        _handler: &'a Arc<dyn WebSocketHandler>,
    ) -> BeforeHandshakeFuture<'a> {
        Box::pin(async { Ok(true) })
    }

    /// 握手结束（成功或失败）后调用。failure 提供可能的错误。
    fn after_handshake<'a>(
        &'a self,
        _context: &'a mut HandshakeContext,
        _handler: &'a Arc<dyn WebSocketHandler>,
        _failure: Option<&'a WebSocketError>,
    ) -> AfterHandshakeFuture<'a> {
        Box::pin(async {})
    }
}

/// 把 handler trait 的 Future 类型用于拦截器实现包装。
pub type HandlerInterceptorFuture<'a, T = ()> = HandlerFuture<'a, T>;
