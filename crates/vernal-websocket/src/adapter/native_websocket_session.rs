//! 对应 Java 类：org.springframework.web.socket.adapter.NativeWebSocketSession
//!
//! 暴露底层原生 session 的 SPI。Spring 用 `Object` + 反射，Rust 用 associated type
//! 与 `Any`。Rust 化的等价语义：adapter 必须能返回原生 session 引用或按需 downcast。

use crate::WebSocketSession;

/// 可暴露原生 session 的 trait。
pub trait NativeWebSocketSession: WebSocketSession {
    /// 原生 session 的关联类型。
    type Native: std::any::Any + Send + Sync;

    /// 返回原生 session 引用。
    fn native_session(&self) -> &Self::Native;
}
