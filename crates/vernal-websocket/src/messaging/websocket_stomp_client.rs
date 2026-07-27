//! 对应 Java 类：org.springframework.web.socket.messaging.WebSocketStompClient
//! 与 spring-messaging StompSession（合并入此模块）
//!
//! 客户端 STOMP session：通过 WebSocketClient 建立 WS 连接，发送 CONNECT，
//! 等待 CONNECTED，管理订阅/receipt/heartbeat/disconnect。

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use vernal_messaging::simp_headers;

use crate::stomp::stomp_codec::{StompDecoder, StompEncoder, StompFrame};
use crate::stomp::stomp_command::StompCommand;
use crate::stomp::stomp_headers::{StompHeaders, headers};
use crate::{CloseCode, CloseStatus, WebSocketError, WebSocketMessage};

/// STOMP 客户端 session 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StompSessionState {
    /// 已连接 WebSocket 但 STOMP 未握手。
    Connecting,
    /// 已收到 CONNECTED。
    Connected,
    /// 已发送 DISCONNECT。
    Disconnecting,
    /// 已关闭。
    Closed,
}

/// 订阅句柄。
#[derive(Debug, Clone)]
pub struct StompSubscription {
    /// 订阅 id。
    pub id: String,
    /// destination。
    pub destination: String,
}

/// STOMP 客户端 session。
pub struct StompSession {
    state: Mutex<StompSessionState>,
    subscriptions: Mutex<BTreeMap<String, StompSubscription>>,
    decoder: StompDecoder,
    encoder: StompEncoder,
    heartbeat: Mutex<Option<Duration>>,
}

impl StompSession {
    /// 创建 session。
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(StompSessionState::Connecting),
            subscriptions: Mutex::new(BTreeMap::new()),
            decoder: StompDecoder::new(),
            encoder: StompEncoder::new(),
            heartbeat: Mutex::new(None),
        }
    }

    /// 构造 CONNECT 帧。
    pub fn connect_frame(
        &self,
        accept_version: &str,
        host: &str,
        heartbeat: Option<(u32, u32)>,
    ) -> Result<WebSocketMessage, WebSocketError> {
        let mut headers = StompHeaders::new();
        headers.set(headers::ACCEPT_VERSION, accept_version);
        headers.set(headers::HOST, host);
        if let Some((send, receive)) = heartbeat {
            headers.set(headers::HEART_BEAT, format!("{send},{receive}"));
        }
        let frame = StompFrame {
            command: StompCommand::Connect,
            headers,
            body: bytes::Bytes::new(),
        };
        let encoded = self.encoder.encode(&frame);
        Ok(WebSocketMessage::text(
            String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "encode failure")
            })?,
        ))
    }

    /// 处理来自服务端的 WebSocket 消息；返回需要回送的帧列表。
    ///
    /// # Errors
    ///
    /// 解析失败时返回错误。
    pub async fn handle_message(
        &self,
        message: &WebSocketMessage,
    ) -> Result<Vec<WebSocketMessage>, WebSocketError> {
        let payload = match message {
            WebSocketMessage::Text(text) => text.as_bytes().to_vec(),
            WebSocketMessage::Binary(bytes) => bytes.to_vec(),
            _ => return Ok(Vec::new()),
        };
        let frames = self.decoder.decode(&payload)?;
        let mut responses = Vec::new();
        for frame in frames {
            match frame.command {
                StompCommand::Connected => {
                    let mut state = self.state.lock().await;
                    if *state != StompSessionState::Connecting {
                        return Err(WebSocketError::protocol(
                            CloseCode::ProtocolError,
                            "Unexpected CONNECTED",
                        ));
                    }
                    *state = StompSessionState::Connected;
                    let heartbeat = frame
                        .headers
                        .get(headers::HEART_BEAT)
                        .and_then(|value| parse_heartbeat(value));
                    *self.heartbeat.lock().await = heartbeat;
                }
                StompCommand::Message => {
                    // 应用层处理；此处仅记录订阅匹配
                    if let Some(subscription_id) = frame.headers.subscription() {
                        let subscriptions = self.subscriptions.lock().await;
                        if !subscriptions.contains_key(subscription_id) {
                            return Err(WebSocketError::protocol(
                                CloseCode::ProtocolError,
                                format!("Unknown subscription: {subscription_id}"),
                            ));
                        }
                    }
                }
                StompCommand::Receipt => {
                    // receipt 处理；简化为状态推进
                }
                StompCommand::Error => {
                    let reason = frame
                        .headers
                        .get(headers::MESSAGE)
                        .unwrap_or("STOMP error")
                        .to_string();
                    *self.state.lock().await = StompSessionState::Closed;
                    return Err(WebSocketError::protocol(CloseCode::PolicyViolation, reason));
                }
                StompCommand::Disconnect => {
                    *self.state.lock().await = StompSessionState::Closed;
                }
                _ => {}
            }
        }
        Ok(responses)
    }

    /// 构造 SUBSCRIBE 帧。
    pub async fn subscribe(
        &self,
        subscription_id: impl Into<String>,
        destination: impl Into<String>,
    ) -> Result<WebSocketMessage, WebSocketError> {
        let subscription_id = subscription_id.into();
        let destination = destination.into();
        let mut headers = StompHeaders::new();
        headers.set(headers::ID, subscription_id.clone());
        headers.set(headers::DESTINATION, destination.clone());
        headers.set("ack", "auto");
        let frame = StompFrame {
            command: StompCommand::Subscribe,
            headers,
            body: bytes::Bytes::new(),
        };
        let encoded = self.encoder.encode(&frame);
        self.subscriptions.lock().await.insert(
            subscription_id,
            StompSubscription {
                id: frame
                    .headers
                    .get(headers::ID)
                    .unwrap_or_default()
                    .to_string(),
                destination,
            },
        );
        Ok(WebSocketMessage::text(
            String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "encode failure")
            })?,
        ))
    }

    /// 构造 UNSUBSCRIBE 帧。
    pub async fn unsubscribe(
        &self,
        subscription_id: &str,
    ) -> Result<WebSocketMessage, WebSocketError> {
        let mut headers = StompHeaders::new();
        headers.set(headers::ID, subscription_id);
        let frame = StompFrame {
            command: StompCommand::Unsubscribe,
            headers,
            body: bytes::Bytes::new(),
        };
        let encoded = self.encoder.encode(&frame);
        self.subscriptions.lock().await.remove(subscription_id);
        Ok(WebSocketMessage::text(
            String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "encode failure")
            })?,
        ))
    }

    /// 构造 SEND 帧。
    pub fn send_frame(
        &self,
        destination: &str,
        body: &[u8],
    ) -> Result<WebSocketMessage, WebSocketError> {
        let mut headers = StompHeaders::new();
        headers.set(headers::DESTINATION, destination);
        headers.set(headers::CONTENT_LENGTH, body.len().to_string());
        let frame = StompFrame {
            command: StompCommand::Send,
            headers,
            body: bytes::Bytes::copy_from_slice(body),
        };
        let encoded = self.encoder.encode(&frame);
        Ok(WebSocketMessage::text(
            String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "encode failure")
            })?,
        ))
    }

    /// 构造 DISCONNECT 帧。
    pub fn disconnect_frame(&self, receipt_id: &str) -> Result<WebSocketMessage, WebSocketError> {
        let mut headers = StompHeaders::new();
        headers.set(headers::RECEIPT, receipt_id);
        let frame = StompFrame {
            command: StompCommand::Disconnect,
            headers,
            body: bytes::Bytes::new(),
        };
        let encoded = self.encoder.encode(&frame);
        Ok(WebSocketMessage::text(
            String::from_utf8(encoded.to_vec()).map_err(|_| {
                WebSocketError::protocol(CloseCode::ProtocolError, "encode failure")
            })?,
        ))
    }

    /// 返回当前状态。
    pub async fn state(&self) -> StompSessionState {
        *self.state.lock().await
    }

    /// 返回协商出心跳间隔。
    pub async fn heartbeat(&self) -> Option<Duration> {
        *self.heartbeat.lock().await
    }

    /// 返回订阅数。
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.lock().await.len()
    }

    /// 提取 SIMP headers（用于 MessageHeaderAccessor 互操作）。
    pub fn to_simp_headers(
        &self,
        session_id: &str,
        message_type: &str,
    ) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert(
            simp_headers::SIMP_SESSION_ID.to_string(),
            session_id.to_string(),
        );
        map.insert(
            simp_headers::SIMP_MESSAGE_TYPE.to_string(),
            message_type.to_string(),
        );
        map
    }
}

impl Default for StompSession {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_heartbeat(value: &str) -> Option<Duration> {
    let (send, receive): (u32, u32) = value
        .split_once(',')
        .and_then(|(s, r)| Some((s.parse().ok()?, r.parse().ok()?)))?;
    let max = send.max(receive);
    if max == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(max)))
    }
}

/// STOMP 客户端。对标 Spring `WebSocketStompClient`。
pub struct WebSocketStompClient {
    /// 目标 URL。
    pub url: String,
    /// session。
    pub session: Arc<StompSession>,
}

impl WebSocketStompClient {
    /// 创建客户端。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            session: Arc::new(StompSession::new()),
        }
    }

    /// 生成初始 CONNECT 帧。
    pub fn connect(&self, host: &str) -> Result<WebSocketMessage, WebSocketError> {
        self.session
            .connect_frame("1.2", host, Some((10000, 10000)))
    }
}

/// 关闭状态辅助：把 StompSession 关闭转为 CloseStatus。
#[must_use]
pub fn stomp_close_status(error: &WebSocketError) -> CloseStatus {
    match error {
        WebSocketError::Protocol { .. } => {
            CloseStatus::new(CloseCode::ProtocolError, "STOMP protocol error")
                .unwrap_or_else(|_| CloseStatus::normal())
        }
        _ => CloseStatus::new(CloseCode::ServerError, "STOMP client error")
            .unwrap_or_else(|_| CloseStatus::normal()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_to_connected_state_transition() {
        let session = StompSession::new();
        assert_eq!(session.state().await, StompSessionState::Connecting);
        let connected = "CONNECTED\nversion:1.2\nheart-beat:10000,10000\n\n\x00";
        let _ = session
            .handle_message(&WebSocketMessage::text(connected))
            .await
            .unwrap();
        assert_eq!(session.state().await, StompSessionState::Connected);
        assert!(session.heartbeat().await.is_some());
    }

    #[tokio::test]
    async fn subscribe_and_unsubscribe_track_state() {
        let session = StompSession::new();
        let _ = session.subscribe("sub-1", "/topic/test").await.unwrap();
        assert_eq!(session.subscription_count().await, 1);
        let _ = session.unsubscribe("sub-1").await.unwrap();
        assert_eq!(session.subscription_count().await, 0);
    }

    #[test]
    fn send_frame_contains_destination_and_body() {
        let session = StompSession::new();
        let frame = session.send_frame("/queue/test", b"hello").unwrap();
        if let WebSocketMessage::Text(text) = frame {
            assert!(text.contains("SEND"));
            assert!(text.contains("destination:/queue/test"));
            assert!(text.contains("hello"));
        } else {
            panic!("expected text frame");
        }
    }

    #[tokio::test]
    async fn error_frame_closes_session() {
        let session = StompSession::new();
        let error_frame = "ERROR\nmessage:forbidden\n\n\x00";
        let result = session
            .handle_message(&WebSocketMessage::text(error_frame))
            .await;
        assert!(result.is_err());
        assert_eq!(session.state().await, StompSessionState::Closed);
    }
}
