//! 对应 Java 类：org.springframework.web.socket.messaging.SubProtocolErrorHandler
//!
//! 子协议错误处理 SPI。

use std::{future::Future, pin::Pin};

use crate::WebSocketError;

/// 错误处理返回的字节 future。
pub type SubProtocolErrorFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<u8>, WebSocketError>> + Send + 'a>>;

/// 子协议错误处理 SPI。
pub trait SubProtocolErrorHandler<P: Send + Sync>: Send + Sync {
    /// 把应用错误转换为协议层错误 payload。
    fn handle_error<'a>(&'a self, error: &'a P) -> SubProtocolErrorFuture<'a>;
}
