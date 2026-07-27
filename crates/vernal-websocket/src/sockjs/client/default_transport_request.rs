//! 对应 Java 类：org.springframework.web.socket.sockjs.client.DefaultTransportRequest
//!
//! TransportRequest 默认实现。

use std::sync::Arc;

use crate::sockjs::client::sockjs_url_info::SockJsUrlInfo;
use crate::sockjs::client::transport::TransportRequest;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use crate::sockjs::transport::transport_type::TransportType;

/// 默认 transport 请求。
pub struct DefaultTransportRequest {
    url_info: SockJsUrlInfo,
    handshake_headers: http::HeaderMap,
    http_headers: http::HeaderMap,
    transport_type: TransportType,
    codec: Arc<dyn SockJsMessageCodec>,
}

impl DefaultTransportRequest {
    /// 创建请求。
    #[must_use]
    pub fn new(
        url_info: SockJsUrlInfo,
        transport_type: TransportType,
        codec: Arc<dyn SockJsMessageCodec>,
    ) -> Self {
        Self {
            url_info,
            handshake_headers: http::HeaderMap::new(),
            http_headers: http::HeaderMap::new(),
            transport_type,
            codec,
        }
    }

    /// 设置握手 headers。
    #[must_use]
    pub fn with_handshake_headers(mut self, headers: http::HeaderMap) -> Self {
        self.handshake_headers = headers;
        self
    }

    /// 设置 HTTP headers。
    #[must_use]
    pub fn with_http_headers(mut self, headers: http::HeaderMap) -> Self {
        self.http_headers = headers;
        self
    }
}

impl TransportRequest for DefaultTransportRequest {
    fn sockjs_url_info(&self) -> &SockJsUrlInfo {
        &self.url_info
    }
    fn handshake_headers(&self) -> &http::HeaderMap {
        &self.handshake_headers
    }
    fn http_request_headers(&self) -> &http::HeaderMap {
        &self.http_headers
    }
    fn transport_url(&self) -> String {
        self.url_info.transport_url(self.transport_type)
    }
    fn message_codec(&self) -> Arc<dyn SockJsMessageCodec> {
        Arc::clone(&self.codec)
    }
}
