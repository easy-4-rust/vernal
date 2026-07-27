//! 对应 Java 类：org.springframework.web.socket.messaging.StompSubProtocolErrorHandler
//!
//! 把应用层错误编码为 STOMP ERROR 帧。

use crate::WebSocketError;
use crate::messaging::sub_protocol_error_handler::{
    SubProtocolErrorFuture, SubProtocolErrorHandler,
};
use crate::stomp::stomp_codec::{StompEncoder, StompFrame};
use crate::stomp::stomp_command::StompCommand;
use crate::stomp::stomp_headers::{StompHeaders, headers};

/// 字符串错误载荷。
#[derive(Debug, Clone)]
pub struct StompErrorMessage {
    /// 错误消息。
    pub message: String,
}

/// STOMP 错误处理器。
#[derive(Debug, Default)]
pub struct StompSubProtocolErrorHandler;

impl StompSubProtocolErrorHandler {
    /// 创建处理器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl SubProtocolErrorHandler<StompErrorMessage> for StompSubProtocolErrorHandler {
    fn handle_error<'a>(&'a self, error: &'a StompErrorMessage) -> SubProtocolErrorFuture<'a> {
        Box::pin(async move {
            let mut headers = StompHeaders::new();
            headers.set(headers::MESSAGE, error.message.clone());
            headers.set(headers::CONTENT_TYPE, "text/plain");
            let frame = StompFrame {
                command: StompCommand::Error,
                headers,
                body: bytes::Bytes::copy_from_slice(error.message.as_bytes()),
            };
            Ok(StompEncoder::new().encode(&frame).to_vec())
        })
    }
}
