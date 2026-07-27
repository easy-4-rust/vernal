//! Server 模块：握手 SPI、拦截器、handler mapping 与升级策略。

pub mod abstract_handshake_handler;
pub mod default_handshake_handler;
pub mod handshake_failure_error;
pub mod handshake_handler;
pub mod handshake_interceptor;
pub mod handshake_interceptor_chain;
pub mod http_session_handshake_interceptor;
pub mod origin_handshake_interceptor;
pub mod request_upgrade_strategy;
pub mod websocket_handler_mapping;
pub mod websocket_http_request_handler;

pub use abstract_handshake_handler::AbstractHandshakeHandler;
pub use default_handshake_handler::DefaultHandshakeHandler;
pub use handshake_failure_error::HandshakeFailureError;
pub use handshake_handler::{HandshakeFuture, HandshakeHandler};
pub use handshake_interceptor::{
    AfterHandshakeFuture, BeforeHandshakeFuture, HandshakeAttributes, HandshakeContext,
    HandshakeInterceptor,
};
pub use handshake_interceptor_chain::HandshakeInterceptorChain;
pub use http_session_handshake_interceptor::{
    HTTP_SESSION_ID_ATTR_NAME, HttpSession, HttpSessionAttributes, HttpSessionHandshakeInterceptor,
};
pub use origin_handshake_interceptor::OriginHandshakeInterceptor;
pub use request_upgrade_strategy::RequestUpgradeStrategy;
pub use websocket_handler_mapping::{MappingEntry, WebSocketHandlerMapping};
pub use websocket_http_request_handler::{HandshakeResponse, WebSocketHttpRequestHandler};
