//! Adapter 模块：原生 session 抽象。

pub mod abstract_websocket_session;
pub mod native_websocket_session;

pub use abstract_websocket_session::{AbstractWebSocketSession, ensure_control_payload};
pub use native_websocket_session::NativeWebSocketSession;
