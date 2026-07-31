//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.handler.AbstractHttpReceivingTransportHandler
//! 与 XhrReceivingTransportHandler / XhrSendTransportHandler
//!
//! HTTP POST 接收端：读取 body → 解码 SockJS 消息 → delegateMessages → 返回响应。
//! 对标 Spring `AbstractHttpReceivingTransportHandler` + `XhrReceivingTransportHandler` + `XhrSendTransportHandler`。

use std::sync::Arc;

use crate::sockjs::transport::TransportType;
use crate::sockjs::transport::handler::abstract_transport_handler::AbstractTransportHandler;
use crate::sockjs::transport::transport_handler::{TransportHandleFuture, TransportHandler};

/// HTTP POST 接收端 handler。合并 Spring `AbstractHttpReceivingTransportHandler`
/// + `XhrReceivingTransportHandler` + `XhrSendTransportHandler` 的语义。
pub struct HttpReceivingTransportHandler {
    base: AbstractTransportHandler,
    transport_type: TransportType,
}

impl HttpReceivingTransportHandler {
    /// 创建 XHR send handler（POST /xhr_send）。
    #[must_use]
    pub fn xhr_send() -> Self {
        Self {
            base: AbstractTransportHandler::new(),
            transport_type: TransportType::XhrSend,
        }
    }

    /// 创建 XHR receive handler（POST /xhr 用于接收）。
    #[must_use]
    pub fn xhr() -> Self {
        Self {
            base: AbstractTransportHandler::new(),
            transport_type: TransportType::Xhr,
        }
    }

    /// 处理 POST body：解码消息 → delegateMessages → 返回 "ok" 响应。
    ///
    /// # Errors
    ///
    /// 当 body 无法解码或 session 不匹配时返回错误。
    pub async fn handle_post(
        &self,
        body: &[u8],
        session: &crate::sockjs::transport::session::AbstractSockJsSession,
    ) -> Result<Vec<u8>, crate::sockjs::SockJsError> {
        let config = self.base.config().ok_or_else(|| {
            crate::sockjs::SockJsError::new("TransportHandler not initialized", None, None)
        })?;
        let text = std::str::from_utf8(body)
            .map_err(|_| crate::sockjs::SockJsError::new("Broken JSON encoding", None, None))?;
        let messages = config.message_codec.decode(text).map_err(|err| {
            crate::sockjs::SockJsError::new(format!("Failed to read message(s): {err}"), None, None)
        })?;
        if messages.is_empty() {
            return Err(crate::sockjs::SockJsError::new(
                "Payload expected.",
                Some(session.id().to_string()),
                None,
            ));
        }
        session.delegate_messages(&messages).await;
        Ok(b"ok".to_vec())
    }
}

impl TransportHandler for HttpReceivingTransportHandler {
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
