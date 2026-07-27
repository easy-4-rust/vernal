#![forbid(unsafe_code)]
#![doc = "Vernal WebSocket 支持（对标 spring-websocket）。"]

pub mod adapter;
pub mod client;
pub mod handler;
pub mod messaging;
pub mod server;
pub mod sockjs;
pub mod stomp;

mod backpressure;
mod close_status;
mod config;
mod error;
mod handshake;
mod lifecycle;
mod message;
mod registry;
mod session;
mod sub_protocol;
mod transport;
mod websocket_extension;
mod websocket_http_headers;

pub use backpressure::{BackpressurePolicy, OutboundQueue};
pub use close_status::{CloseCode, CloseStatus, CloseStatusError, MAX_CLOSE_REASON_BYTES};
pub use config::WebSocketConfig;
pub use error::WebSocketError;
pub use handler::{HandlerFuture, WebSocketHandler};
pub use handshake::{HandshakeRequest, OriginPolicy, header_value};
pub use lifecycle::{LifecycleController, LifecycleState};
pub use message::{MessageKind, WebSocketMessage};
pub use registry::WebSocketRegistry;
pub use session::{
    MemoryWebSocketSession, SessionState, WebSocketSession, normalize_close_code, payload_bytes,
};
pub use sub_protocol::{SubProtocolCapable, is_valid_subprotocol, negotiate_subprotocol};
pub use transport::{TokioWebsocketsTransport, WebSocketTransport};
#[cfg(feature = "tokio-websockets")]
pub use transport::{from_tokio_message, to_tokio_message};
pub use websocket_extension::WebSocketExtension;
pub use websocket_http_headers::WebSocketHttpHeaders;
