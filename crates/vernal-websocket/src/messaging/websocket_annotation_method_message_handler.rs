//! 对应 Java 类：org.springframework.web.socket.messaging.WebSocketAnnotationMethodMessageHandler
//!
//! Spring 用反射 + 注解路由 @MessageMapping 方法；Rust 无运行时反射，
//! 改为显式 destination → handler 闭包注册表。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use vernal_messaging::{Message, MessageChannel, MessageHandler, SendFuture};

use crate::WebSocketError;

/// 显式 destination handler 闭包。
pub type DestinationHandler = Arc<dyn Fn(Arc<dyn Message>) -> SendFuture<'static> + Send + Sync>;

/// destination 前缀路由处理器。对标 `WebSocketAnnotationMethodMessageHandler`。
pub struct WebSocketAnnotationMethodMessageHandler {
    handlers: Mutex<HashMap<String, DestinationHandler>>,
    /// 入站消息通道（对标 Spring `inboundChannel`，暂未使用）。
    #[allow(dead_code)]
    inbound_channel: Arc<dyn MessageChannel>,
}

impl WebSocketAnnotationMethodMessageHandler {
    /// 创建 handler。
    #[must_use]
    pub fn new(inbound_channel: Arc<dyn MessageChannel>) -> Self {
        Self {
            handlers: Mutex::new(HashMap::new()),
            inbound_channel,
        }
    }

    /// 注册 destination handler。
    pub async fn register(&self, destination: impl Into<String>, handler: DestinationHandler) {
        self.handlers
            .lock()
            .await
            .insert(destination.into(), handler);
    }

    /// 返回已注册 destination 数量。
    pub async fn destination_count(&self) -> usize {
        self.handlers.lock().await.len()
    }
}

impl MessageHandler for WebSocketAnnotationMethodMessageHandler {
    fn handle_message<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a> {
        Box::pin(async move {
            let headers = message.headers();
            let Some(destination) = headers.get(vernal_messaging::simp_headers::SIMP_DESTINATION)
            else {
                return Ok(());
            };
            let handlers = self.handlers.lock().await;
            let Some(handler) = handlers.get(destination).cloned() else {
                return Ok(());
            };
            drop(handlers);
            handler(Arc::clone(&message)).await
        })
    }
}

/// 错误：未注册 destination。
pub fn missing_destination_error(destination: &str) -> WebSocketError {
    WebSocketError::protocol(
        crate::CloseCode::ServerError,
        format!("No handler registered for destination: {destination}"),
    )
}
