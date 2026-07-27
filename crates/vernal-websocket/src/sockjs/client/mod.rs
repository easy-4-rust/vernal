//! SockJS client 子模块。

pub mod abstract_client_sockjs_session;
pub mod abstract_xhr_transport;
pub mod default_transport_request;
pub mod info_receiver;
pub mod sockjs_client;
pub mod sockjs_url_info;
pub mod transport;

pub use abstract_client_sockjs_session::{ClientSessionState, ClientSockJsSession};
pub use abstract_xhr_transport::{HttpRequestExecutor, WebSocketClientTransport, XhrTransportImpl};
pub use default_transport_request::DefaultTransportRequest;
pub use info_receiver::{InfoFuture, InfoReceiver, ServerInfo, parse_info_json};
pub use sockjs_client::SockJsClient;
pub use sockjs_url_info::SockJsUrlInfo;
pub use transport::{Transport, TransportConnectFuture, TransportRequest};
