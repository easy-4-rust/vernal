//! 对应 Java 类：org.springframework.web.socket.server.support.OriginHandshakeInterceptor
//!
//! 检查 `Origin` header 是否在允许列表中。对标 Spring
//! `OriginHandshakeInterceptor`：默认禁止跨域；可通过 `with_allowed_origins`
//! 显式配置允许的 origin；`*` 视为允许所有。

use std::collections::BTreeSet;

use crate::{
    WebSocketError,
    server::{HandshakeContext, HandshakeInterceptor},
};

/// Origin 握手拦截器。
#[derive(Debug, Clone, Default)]
pub struct OriginHandshakeInterceptor {
    allowed: BTreeSet<String>,
}

impl OriginHandshakeInterceptor {
    /// 创建默认拦截器（无允许 origin，禁止跨域请求）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加允许的 origin（精确匹配或 `*`）。
    #[must_use]
    pub fn with_allowed_origins(mut self, origins: impl IntoIterator<Item = String>) -> Self {
        self.allowed = origins.into_iter().collect();
        self
    }

    fn is_allowed(&self, origin: &str) -> bool {
        self.allowed
            .iter()
            .any(|allowed| allowed == "*" || allowed == origin)
    }
}

impl HandshakeInterceptor for OriginHandshakeInterceptor {
    fn before_handshake<'a>(
        &'a self,
        context: &'a mut HandshakeContext,
        _handler: &'a std::sync::Arc<dyn crate::WebSocketHandler>,
    ) -> crate::server::BeforeHandshakeFuture<'a> {
        Box::pin(async move {
            // Spring WebUtils.isSameOrigin(request) 等价：当 Origin 缺失或同源时允许。
            let Some(origin) = context
                .headers
                .get("origin")
                .and_then(|value| value.to_str().ok())
            else {
                return Ok(true);
            };
            if self.is_allowed(origin) {
                Ok(true)
            } else {
                Err(WebSocketError::Handshake {
                    status: 403,
                    reason: format!("Origin 不允许: {origin}"),
                })
            }
        })
    }
}
