//! 对应 Java 类：org.springframework.web.socket.handler.BeanCreatingHandlerProvider
//!
//! Spring 使用 `BeanFactory` 创建/销毁 handler。Rust 无 `IoC` 容器，
//! 通过 `HandlerFactory` 闭包表达等价能力。

use std::sync::Arc;

use crate::{WebSocketError, WebSocketHandler};

/// Handler 工厂闭包。
pub type HandlerFactory = Arc<dyn Fn() -> Arc<dyn WebSocketHandler> + Send + Sync>;

/// 通过工厂创建 handler 的提供者。
#[derive(Clone)]
pub struct BeanCreatingHandlerProvider {
    factory: HandlerFactory,
    destructor: Arc<dyn Fn(&Arc<dyn WebSocketHandler>) + Send + Sync>,
}

impl BeanCreatingHandlerProvider {
    /// 创建 provider。
    #[must_use]
    pub fn new(factory: HandlerFactory) -> Self {
        Self {
            factory,
            destructor: Arc::new(|_| {}),
        }
    }

    /// 创建带析构钩子的 provider。
    #[must_use]
    pub fn with_destructor(
        factory: HandlerFactory,
        destructor: Arc<dyn Fn(&Arc<dyn WebSocketHandler>) + Send + Sync>,
    ) -> Self {
        Self {
            factory,
            destructor,
        }
    }

    /// 创建新 handler。
    #[must_use]
    pub fn get_handler(&self) -> Arc<dyn WebSocketHandler> {
        (self.factory)()
    }

    /// 销毁 handler（Spring destroyBean 的等价钩子）。
    pub fn destroy(&self, handler: &Arc<dyn WebSocketHandler>) {
        (self.destructor)(handler);
    }
}

impl std::fmt::Debug for BeanCreatingHandlerProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BeanCreatingHandlerProvider")
            .finish_non_exhaustive()
    }
}

/// 等价 Spring `BeanCreatingHandlerProvider` 在创建失败时的错误。
#[derive(Debug)]
pub struct HandlerCreationError {
    /// 错误原因。
    pub reason: String,
}

impl std::fmt::Display for HandlerCreationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for HandlerCreationError {}

impl From<HandlerCreationError> for WebSocketError {
    fn from(error: HandlerCreationError) -> Self {
        WebSocketError::protocol(crate::CloseCode::ServerError, error.reason)
    }
}
