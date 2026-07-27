//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.handler.AbstractHttpSendingTransportHandler
//! 与 XhrPollingTransportHandler / XhrStreamingTransportHandler / EventSourceTransportHandler / HtmlFileTransportHandler
//!
//! HTTP 推送端：按 transport 格式发送 SockJS 帧（open/heartbeat/message/close）。
//! 对标 Spring 4 个 transport handler 的核心行为。

use std::sync::Arc;

use crate::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
use crate::sockjs::frame::sockjs_frame::SockJsFrame;
use crate::sockjs::frame::sockjs_frame_format::SockJsFrameFormat;
use crate::sockjs::transport::TransportType;
use crate::sockjs::transport::handler::abstract_transport_handler::AbstractTransportHandler;
use crate::sockjs::transport::session::abstract_http_sockjs_session::HttpSockJsSession;
use crate::sockjs::transport::transport_handler::{TransportHandleFuture, TransportHandler};

/// HTTP 推送 handler。合并 Spring `AbstractHttpSendingTransportHandler`
/// + XhrPolling/XhrStreaming/EventSource/HtmlFile 的行为。
pub struct HttpSendingTransportHandler {
    base: AbstractTransportHandler,
    transport_type: TransportType,
}

impl HttpSendingTransportHandler {
    /// 创建指定 transport 类型的 handler。
    #[must_use]
    pub fn new(transport_type: TransportType) -> Self {
        Self {
            base: AbstractTransportHandler::new(),
            transport_type,
        }
    }

    /// 创建 XHR polling handler。
    #[must_use]
    pub fn xhr_polling() -> Self {
        Self::new(TransportType::Xhr)
    }

    /// 创建 XHR streaming handler。
    #[must_use]
    pub fn xhr_streaming() -> Self {
        Self::new(TransportType::XhrStreaming)
    }

    /// 创建 EventSource handler。
    #[must_use]
    pub fn event_source() -> Self {
        Self::new(TransportType::EventSource)
    }

    /// 创建 HtmlFile handler。
    #[must_use]
    pub fn html_file() -> Self {
        Self::new(TransportType::HtmlFile)
    }

    /// 返回 content-type。
    #[must_use]
    pub fn content_type(&self) -> &'static str {
        match self.transport_type {
            TransportType::Xhr | TransportType::XhrStreaming => {
                "application/javascript; charset=UTF-8"
            }
            TransportType::EventSource => "text/event-stream; charset=UTF-8",
            TransportType::HtmlFile => "text/html; charset=UTF-8",
            _ => "text/plain; charset=UTF-8",
        }
    }

    /// 返回帧格式化器。
    fn frame_format(&self) -> Box<dyn SockJsFrameFormat> {
        match self.transport_type {
            TransportType::EventSource => Box::new(EventSourceFormat),
            TransportType::HtmlFile => Box::new(DefaultSockJsFrameFormat),
            _ => Box::new(DefaultSockJsFrameFormat),
        }
    }

    /// 处理发送请求：首次发 open + 缓存消息；后续发消息或心跳。
    pub async fn handle_send(
        &self,
        session: &HttpSockJsSession,
    ) -> Result<(String, Vec<u8>), crate::sockjs::SockJsError> {
        let format = self.frame_format();
        let bytes = if session.base().is_new().await {
            session.handle_initial_request(format.as_ref()).await
        } else if session.base().is_closed().await {
            let frame = SockJsFrame::close_frame_go_away();
            format.format(&frame).into_bytes()
        } else if !session.is_active().await {
            session.handle_successive_request(format.as_ref()).await
        } else {
            let frame = SockJsFrame::close_frame_another_connection_open();
            format.format(&frame).into_bytes()
        };
        Ok((self.content_type().to_string(), bytes))
    }
}

impl TransportHandler for HttpSendingTransportHandler {
    fn initialize(&self, config: crate::sockjs::transport::SockJsServiceConfig) {
        self.base.set_config(config);
    }
    fn transport_type(&self) -> TransportType {
        self.transport_type
    }
    fn check_session_type(
        &self,
        _session: &dyn crate::sockjs::transport::sockjs_session::SockJsSession,
    ) -> bool {
        true
    }
    fn handle_request(
        &self,
        _handler: Arc<dyn crate::WebSocketHandler>,
        _session: Arc<dyn crate::sockjs::transport::sockjs_session::SockJsSession>,
    ) -> TransportHandleFuture {
        Box::pin(async { Ok(()) })
    }
}

/// EventSource 帧格式：`data: {content}\n\n`。
struct EventSourceFormat;

impl SockJsFrameFormat for EventSourceFormat {
    fn format(&self, frame: &SockJsFrame) -> String {
        format!("data: {}\n\n", frame.content())
    }
}
