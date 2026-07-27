//! 对应 Java 类：org.springframework.web.socket.handler.WebSocketHandlerDecoratorFactory
//!
//! 把装饰器应用到一个 `WebSocketHandler` 的工厂 SPI。

use std::sync::Arc;

use crate::WebSocketHandler;

/// Handler 装饰器工厂 trait。
pub trait WebSocketHandlerDecoratorFactory: Send + Sync {
    /// 装饰 handler；返回新的或原 handler。
    fn decorate(&self, handler: Arc<dyn WebSocketHandler>) -> Arc<dyn WebSocketHandler>;
}

/// 恒等装饰工厂，原样返回。
#[derive(Default, Debug, Clone, Copy)]
pub struct IdentityHandlerDecoratorFactory;

impl WebSocketHandlerDecoratorFactory for IdentityHandlerDecoratorFactory {
    fn decorate(&self, handler: Arc<dyn WebSocketHandler>) -> Arc<dyn WebSocketHandler> {
        handler
    }
}
