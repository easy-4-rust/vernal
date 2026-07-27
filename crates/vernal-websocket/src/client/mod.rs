//! Client 模块：客户端 SPI、连接管理与抽象基类。

pub mod abstract_websocket_client;
pub mod connection_manager_support;
pub mod websocket_client;

pub use abstract_websocket_client::{AbstractWebSocketClient, delegate_execute};
pub use connection_manager_support::{
    ConnectionManagerSupport, WebSocketClientConfig, WebSocketConnectionManager,
};
pub use websocket_client::{ConnectFuture, WebSocketClient};
