//! 对应 Java 类：org.springframework.web.socket.server.support.HandshakeInterceptorChain
//!
//! 协助顺序执行 `HandshakeInterceptor` 列表：before 成功才推进 index；
//! before 失败时回退触发已执行拦截器的 after；after 按 reverse 顺序调用，
//! 且 after 的异常仅记录，不向上传播。

use std::sync::Arc;

use crate::{WebSocketError, WebSocketHandler, server::HandshakeContext};

/// 拦截器链。
pub struct HandshakeInterceptorChain {
    interceptors: Vec<Arc<dyn crate::server::HandshakeInterceptor>>,
    handler: Arc<dyn WebSocketHandler>,
}

impl HandshakeInterceptorChain {
    /// 创建拦截器链。
    #[must_use]
    pub fn new(
        interceptors: Vec<Arc<dyn crate::server::HandshakeInterceptor>>,
        handler: Arc<dyn WebSocketHandler>,
    ) -> Self {
        Self {
            interceptors,
            handler,
        }
    }

    /// 按顺序执行 beforeHandshake；任一返回 false 时回退触发已执行拦截器的 after。
    ///
    /// # Errors
    ///
    /// 任意拦截器抛错时立即返回错误，并触发 afterHandshake 回滚。
    pub async fn apply_before_handshake(
        &self,
        context: &mut HandshakeContext,
    ) -> Result<bool, WebSocketError> {
        for (index, interceptor) in self.interceptors.iter().enumerate() {
            let proceed = interceptor.before_handshake(context, &self.handler).await?;
            if !proceed {
                tracing::debug!(index, "HandshakeInterceptor 返回 false，终止握手");
                self.apply_after_handshake(context, None, index).await;
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 逆序执行 afterHandshake，吞掉每个拦截器的异常（Spring 行为：仅记录 warn）。
    pub async fn apply_after_handshake(
        &self,
        context: &mut HandshakeContext,
        failure: Option<&WebSocketError>,
        executed_count: usize,
    ) {
        for interceptor in self.interceptors[..executed_count].iter().rev() {
            let () = interceptor
                .after_handshake(context, &self.handler, failure)
                .await;
        }
    }
}
