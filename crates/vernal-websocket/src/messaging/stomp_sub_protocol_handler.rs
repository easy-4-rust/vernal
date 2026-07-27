//! 对应 Java 类：org.springframework.web.socket.messaging.StompSubProtocolHandler
//!
//! 把 STOMP over WebSocket 消息与 SIMP `Message` 互转。
//! 对标 Spring `StompSubProtocolHandler` 的核心行为：
//! - 客户端消息：解码 STOMP 帧 → 构造 SIMP Message（含 simpSessionId/destination 等头）→ 发送到 clientInboundChannel
//! - 服务端 MESSAGE 帧：按 destination/subscriptionId 编码为 STOMP MESSAGE 帧并发送给客户端
//! - CONNECT → 等待 CONNECTED；DISCONNECT → 回 RECEIPT；ERROR → 关闭连接
//! - afterSessionEnded：构造 DISCONNECT SIMP Message 发送到 channel

use std::sync::Arc;

use vernal_messaging::SimpMessageType;
use vernal_messaging::{GenericMessage, Message, MessageChannel, simp_headers};

use crate::messaging::sub_protocol_handler::{SubProtocolFuture, SubProtocolHandler};
use crate::stomp::stomp_codec::{StompDecoder, StompEncoder, StompFrame};
use crate::stomp::stomp_command::StompCommand;
use crate::stomp::stomp_headers::{StompHeaders, headers};
use crate::{CloseCode, CloseStatus, WebSocketError, WebSocketMessage, WebSocketSession};

/// STOMP 子协议处理器。
pub struct StompSubProtocolHandler {
    decoder: StompDecoder,
    encoder: StompEncoder,
}

impl StompSubProtocolHandler {
    /// 创建处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            decoder: StompDecoder::new(),
            encoder: StompEncoder::new(),
        }
    }

    fn build_simp_headers(
        session: &dyn WebSocketSession,
        frame: &StompFrame,
    ) -> std::collections::BTreeMap<String, String> {
        let mut headers = std::collections::BTreeMap::new();
        headers.insert(
            simp_headers::SIMP_SESSION_ID.to_string(),
            session.id().to_string(),
        );
        headers.insert(
            simp_headers::SIMP_MESSAGE_TYPE.to_string(),
            match frame.command {
                StompCommand::Stomp | StompCommand::Connect => "CONNECT",
                StompCommand::Disconnect => "DISCONNECT",
                StompCommand::Subscribe => "SUBSCRIBE",
                StompCommand::Unsubscribe => "UNSUBSCRIBE",
                StompCommand::Send | StompCommand::Message => "MESSAGE",
                _ => "OTHER",
            }
            .to_string(),
        );
        if let Some(destination) = frame.headers.destination() {
            headers.insert(
                simp_headers::SIMP_DESTINATION.to_string(),
                destination.to_string(),
            );
        }
        if let Some(subscription) = frame.headers.subscription() {
            headers.insert(
                simp_headers::SIMP_SUBSCRIPTION_ID.to_string(),
                subscription.to_string(),
            );
        }
        if let Some(protocol) = session.accepted_protocol() {
            headers.insert("stompProtocol".to_string(), protocol.to_string());
        }
        headers
    }
}

impl Default for StompSubProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SubProtocolHandler for StompSubProtocolHandler {
    fn supported_protocols(&self) -> Vec<String> {
        vec!["v11.stomp".to_string(), "v12.stomp".to_string()]
    }

    fn handle_message_from_client<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: WebSocketMessage,
        output_channel: Arc<dyn MessageChannel>,
    ) -> SubProtocolFuture<'a> {
        Box::pin(async move {
            let payload = match message {
                WebSocketMessage::Text(text) => text.into_bytes(),
                WebSocketMessage::Binary(bytes) => bytes.to_vec(),
                WebSocketMessage::Ping(_) | WebSocketMessage::Pong(_) => return Ok(()),
                WebSocketMessage::Close(_) => return Ok(()),
                WebSocketMessage::Continuation { payload, .. } => payload.to_vec(),
            };
            let frames = self.decoder.decode(&payload)?;
            for frame in frames {
                let simp_headers = Self::build_simp_headers(session, &frame);
                let simp_message = GenericMessage::new(
                    format!("{}-{}", session.id(), frame.command.as_str()),
                    frame.body.to_vec(),
                );
                let mut with_headers = simp_message;
                for (name, value) in simp_headers {
                    with_headers = with_headers.with_header(name, value);
                }
                if let Err(error) = output_channel.send(Arc::new(with_headers)).await {
                    tracing::warn!(session_id = %session.id(), error = %error, "STOMP inbound channel send failed");
                }
            }
            Ok(())
        })
    }

    fn handle_message_to_client<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        message: Arc<dyn Message>,
    ) -> SubProtocolFuture<'a> {
        Box::pin(async move {
            let headers = message.headers();
            let message_type = headers
                .get(simp_headers::SIMP_MESSAGE_TYPE)
                .cloned()
                .unwrap_or_else(|| SimpMessageType::Other.as_str().to_string());
            let mut stomp_headers = StompHeaders::new();
            if let Some(destination) = headers.get(simp_headers::SIMP_DESTINATION) {
                stomp_headers.set(headers::DESTINATION, destination.clone());
            }
            if let Some(subscription) = headers.get(simp_headers::SIMP_SUBSCRIPTION_ID) {
                stomp_headers.set(headers::SUBSCRIPTION, subscription.clone());
                stomp_headers.set(
                    headers::MESSAGE_ID,
                    format!("{}-{}", session.id(), subscription),
                );
            }
            if let Some(session_id) = headers.get(simp_headers::SIMP_SESSION_ID) {
                let _ = session_id;
            }

            let frame = match message_type.as_str() {
                "CONNECT" => {
                    // 转换为 CONNECTED
                    let mut connected_headers = StompHeaders::new();
                    connected_headers.set(headers::VERSION, "1.2");
                    connected_headers.set(headers::HEART_BEAT, "0,0");
                    StompFrame {
                        command: StompCommand::Connected,
                        headers: connected_headers,
                        body: bytes::Bytes::new(),
                    }
                }
                "DISCONNECT" => {
                    let mut receipt_headers = StompHeaders::new();
                    receipt_headers.set(headers::RECEIPT_ID, "disconnect");
                    StompFrame {
                        command: StompCommand::Receipt,
                        headers: receipt_headers,
                        body: bytes::Bytes::new(),
                    }
                }
                "MESSAGE" => StompFrame {
                    command: StompCommand::Message,
                    headers: stomp_headers,
                    body: bytes::Bytes::copy_from_slice(message.payload()),
                },
                "HEARTBEAT" => return Ok(()),
                _ => {
                    let mut error_headers = StompHeaders::new();
                    error_headers.set(
                        headers::MESSAGE,
                        format!("Unknown message type: {message_type}"),
                    );
                    StompFrame {
                        command: StompCommand::Error,
                        headers: error_headers,
                        body: bytes::Bytes::copy_from_slice(message.payload()),
                    }
                }
            };
            let encoded = self.encoder.encode(&frame);
            let text = String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "STOMP frame not UTF-8")
            })?;
            session.send(WebSocketMessage::text(text)).await?;
            Ok(())
        })
    }

    fn resolve_session_id(&self, message: &dyn Message) -> Option<String> {
        message
            .headers()
            .get(simp_headers::SIMP_SESSION_ID)
            .cloned()
    }

    fn after_session_ended<'a>(
        &'a self,
        session: &'a dyn WebSocketSession,
        close_status: CloseStatus,
        output_channel: Arc<dyn MessageChannel>,
    ) -> SubProtocolFuture<'a> {
        Box::pin(async move {
            let mut headers = std::collections::BTreeMap::new();
            headers.insert(
                simp_headers::SIMP_SESSION_ID.to_string(),
                session.id().to_string(),
            );
            headers.insert(
                simp_headers::SIMP_MESSAGE_TYPE.to_string(),
                SimpMessageType::Disconnect.as_str().to_string(),
            );
            headers.insert(
                "closeStatus".to_string(),
                close_status.code().as_u16().to_string(),
            );
            let disconnect =
                GenericMessage::new(format!("{}-DISCONNECT", session.id()), Vec::new());
            let mut with_headers = disconnect;
            for (name, value) in headers {
                with_headers = with_headers.with_header(name, value);
            }
            let _ = output_channel.send(Arc::new(with_headers)).await;
            Ok(())
        })
    }
}
