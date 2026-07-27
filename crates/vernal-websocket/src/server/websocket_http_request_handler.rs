//! 对应 Java 类：org.springframework.web.socket.server.support.WebSocketHttpRequestHandler
//!
//! HTTP 请求入口：校验握手、执行拦截器、委托 HandshakeHandler、并处理失败响应。
//! 对标 Spring `WebSocketHttpRequestHandler`，返回完整的 HTTP 响应（状态码 + headers + body）。

use std::sync::Arc;

use crate::{
    WebSocketError, WebSocketHandler,
    server::{
        HandshakeContext, HandshakeHandler, HandshakeInterceptorChain,
        handshake_failure_error::HandshakeFailureError,
    },
};

/// HTTP 握手处理结果。
pub struct HandshakeResponse {
    /// HTTP 状态码。
    pub status: u16,
    /// 响应头。
    pub headers: http::HeaderMap,
    /// 响应 body（握手失败时的错误描述）。
    pub body: Vec<u8>,
    /// 成功握手时返回的 WebSocket session。
    pub session: Option<Arc<dyn crate::WebSocketSession>>,
}

impl std::fmt::Debug for HandshakeResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandshakeResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body_len", &self.body.len())
            .field("has_session", &self.session.is_some())
            .finish()
    }
}

impl HandshakeResponse {
    /// 创建成功响应（101 Switching Protocols）。
    #[must_use]
    pub fn switching_protocols(session: Arc<dyn crate::WebSocketSession>) -> Self {
        let mut headers = http::HeaderMap::new();
        if let Ok(v) = "websocket".parse() {
            headers.insert("upgrade", v);
        }
        if let Ok(v) = "Upgrade".parse() {
            headers.insert("connection", v);
        }
        Self {
            status: 101,
            headers,
            body: Vec::new(),
            session: Some(session),
        }
    }

    /// 创建错误响应。
    #[must_use]
    pub fn error(status: u16, reason: &str) -> Self {
        Self {
            status,
            headers: http::HeaderMap::new(),
            body: reason.as_bytes().to_vec(),
            session: None,
        }
    }
}

/// HTTP 请求 handler。
pub struct WebSocketHttpRequestHandler {
    handshake_handler: Arc<dyn HandshakeHandler>,
    interceptors: Vec<Arc<dyn crate::server::HandshakeInterceptor>>,
}

impl WebSocketHttpRequestHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(
        handshake_handler: Arc<dyn HandshakeHandler>,
        interceptors: Vec<Arc<dyn crate::server::HandshakeInterceptor>>,
    ) -> Self {
        Self {
            handshake_handler,
            interceptors,
        }
    }

    /// 处理握手请求，返回完整的 HTTP 响应。
    ///
    /// # Errors
    ///
    /// 握手失败或拦截器终止时返回错误（响应由调用方根据 `HandshakeResponse` 写出）。
    pub async fn handle(
        &self,
        context: &mut HandshakeContext,
        handler: Arc<dyn WebSocketHandler>,
        attributes: std::collections::BTreeMap<String, String>,
    ) -> Result<HandshakeResponse, WebSocketError> {
        let chain = HandshakeInterceptorChain::new(self.interceptors.clone(), Arc::clone(&handler));
        let proceed = chain.apply_before_handshake(context).await?;
        if !proceed {
            chain.apply_after_handshake(context, None, 0).await;
            return Ok(HandshakeResponse::error(
                403,
                "Handshake rejected by interceptor",
            ));
        }
        let request = crate::HandshakeRequest::new(
            context.method.clone(),
            context.uri.clone(),
            context.headers.clone(),
        );
        let outcome = self
            .handshake_handler
            .do_handshake(request, handler, attributes)
            .await;
        match outcome {
            Ok(session) => {
                chain
                    .apply_after_handshake(context, None, self.interceptors.len())
                    .await;
                Ok(HandshakeResponse::switching_protocols(session))
            }
            Err(error) => {
                let reason = error.to_string();
                chain
                    .apply_after_handshake(context, Some(&error), self.interceptors.len())
                    .await;
                let failure: HandshakeFailureError = HandshakeFailureError::new(reason.clone());
                let ws_error: WebSocketError = failure.into();
                let status = match &ws_error {
                    WebSocketError::Handshake { status, .. } => *status,
                    _ => 500,
                };
                Ok(HandshakeResponse::error(status, &reason))
            }
        }
    }
}
