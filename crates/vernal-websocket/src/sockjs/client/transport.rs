//! 对应 Java 类：org.springframework.web.socket.sockjs.client.Transport
//! 与 org.springframework.web.socket.sockjs.client.TransportRequest
//!
//! 客户端 transport SPI。

use std::{future::Future, pin::Pin, sync::Arc};

use crate::WebSocketHandler;
use crate::sockjs::client::sockjs_url_info::SockJsUrlInfo;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use crate::sockjs::transport::transport_type::TransportType;

/// transport 连接返回的 future。
pub type TransportConnectFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

/// 客户端 transport SPI。对标 Spring `Transport`。
pub trait Transport: Send + Sync {
    /// 返回该 transport 支持的 SockJS 传输类型。
    fn transport_types(&self) -> Vec<TransportType>;

    /// 连接到 SockJS 服务端。
    fn connect_async<'a>(
        &'a self,
        request: &'a dyn TransportRequest,
        handler: Arc<dyn WebSocketHandler>,
    ) -> TransportConnectFuture<'a>;
}

/// transport 请求信息。对标 Spring `TransportRequest`。
pub trait TransportRequest: Send + Sync {
    /// 返回 SockJS URL 信息。
    fn sockjs_url_info(&self) -> &SockJsUrlInfo;

    /// 返回握手 headers。
    fn handshake_headers(&self) -> &http::HeaderMap;

    /// 返回 HTTP 请求 headers（非握手）。
    fn http_request_headers(&self) -> &http::HeaderMap;

    /// 返回 transport URL。
    fn transport_url(&self) -> String;

    /// 返回消息 codec。
    fn message_codec(&self) -> Arc<dyn SockJsMessageCodec>;
}
