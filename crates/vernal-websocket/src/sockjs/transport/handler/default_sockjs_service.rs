//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.handler.DefaultSockJsService
//!
//! 默认 SockJS 服务，注册所有 transport handler。
//! 对标 Spring `DefaultSockJsService`。

use std::sync::Arc;

use crate::sockjs::transport::SockJsServiceConfig;
use crate::sockjs::transport::handler::abstract_http_receiving_transport_handler::HttpReceivingTransportHandler;
use crate::sockjs::transport::handler::abstract_http_sending_transport_handler::HttpSendingTransportHandler;
use crate::sockjs::transport::handler::websocket_transport_handler::WebSocketTransportHandler;
use crate::sockjs::transport::transport_handling_sockjs_service::TransportHandlingSockJsService;

/// 创建默认 SockJS 服务（注册全部 transport handler）。
#[must_use]
pub fn default_sockjs_service(config: SockJsServiceConfig) -> TransportHandlingSockJsService {
    let handlers: Vec<Arc<dyn crate::sockjs::transport::transport_handler::TransportHandler>> = vec![
        Arc::new(HttpReceivingTransportHandler::xhr()),
        Arc::new(HttpReceivingTransportHandler::xhr_send()),
        Arc::new(HttpSendingTransportHandler::xhr_polling()),
        Arc::new(HttpSendingTransportHandler::xhr_streaming()),
        Arc::new(HttpSendingTransportHandler::event_source()),
        Arc::new(HttpSendingTransportHandler::html_file()),
        Arc::new(WebSocketTransportHandler::new()),
    ];
    TransportHandlingSockJsService::new(config, handlers)
}
