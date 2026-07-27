//! WebSocket 会话抽象。

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http::{HeaderMap, Uri};
use tokio::sync::RwLock;

use crate::{CloseCode, CloseStatus, WebSocketError, WebSocketMessage};

/// WebSocket 会话状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// 正在连接。
    Connecting,
    /// 已打开。
    Open,
    /// 正在关闭。
    Closing,
    /// 已关闭。
    Closed,
    /// 连接失败。
    Failed,
}

/// WebSocket 会话元数据和异步操作。
pub trait WebSocketSession: Send + Sync {
    /// 返回会话 ID。
    fn id(&self) -> &str;
    /// 返回请求 URI。
    fn uri(&self) -> Option<&Uri>;
    /// 返回握手请求头。
    fn headers(&self) -> &HeaderMap;
    /// 返回当前状态。
    fn state(&self) -> SessionState;
    /// 返回协商出的子协议。
    fn accepted_protocol(&self) -> Option<&str>;
    /// 发送消息。
    fn send(
        &self,
        message: WebSocketMessage,
    ) -> crate::HandlerFuture<'_, Result<(), WebSocketError>>;
    /// 关闭会话。
    fn close(&self, status: CloseStatus) -> crate::HandlerFuture<'_, Result<(), WebSocketError>>;
}

/// 可测试的内存会话。
#[derive(Debug)]
pub struct MemoryWebSocketSession {
    id: String,
    uri: Option<Uri>,
    headers: HeaderMap,
    accepted_protocol: Option<String>,
    state: RwLock<SessionState>,
    sent: RwLock<Vec<WebSocketMessage>>,
    attributes: RwLock<HashMap<String, String>>,
}

impl MemoryWebSocketSession {
    /// 创建已打开的内存会话。
    #[must_use]
    pub fn new(id: impl Into<String>, uri: Option<Uri>, headers: HeaderMap) -> Arc<Self> {
        Arc::new(Self {
            id: id.into(),
            uri,
            headers,
            accepted_protocol: None,
            state: RwLock::new(SessionState::Open),
            sent: RwLock::new(Vec::new()),
            attributes: RwLock::new(HashMap::new()),
        })
    }

    /// 读取已发送消息快照。
    pub async fn sent_messages(&self) -> Vec<WebSocketMessage> {
        self.sent.read().await.clone()
    }

    /// 设置 session attribute。
    pub async fn set_attribute(&self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes
            .write()
            .await
            .insert(key.into(), value.into());
    }

    /// 读取 session attribute。
    pub async fn attribute(&self, key: &str) -> Option<String> {
        self.attributes.read().await.get(key).cloned()
    }
}

impl WebSocketSession for MemoryWebSocketSession {
    fn id(&self) -> &str {
        &self.id
    }
    fn uri(&self) -> Option<&Uri> {
        self.uri.as_ref()
    }
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    fn state(&self) -> SessionState {
        self.state
            .try_read()
            .map_or(SessionState::Open, |state| *state)
    }
    fn accepted_protocol(&self) -> Option<&str> {
        self.accepted_protocol.as_deref()
    }

    fn send(
        &self,
        message: WebSocketMessage,
    ) -> crate::HandlerFuture<'_, Result<(), WebSocketError>> {
        Box::pin(async move {
            if *self.state.read().await != SessionState::Open {
                return Err(WebSocketError::Closed);
            }
            self.sent.write().await.push(message);
            Ok(())
        })
    }

    fn close(&self, status: CloseStatus) -> crate::HandlerFuture<'_, Result<(), WebSocketError>> {
        Box::pin(async move {
            let mut state = self.state.write().await;
            if *state == SessionState::Closed {
                return Ok(());
            }
            *state = SessionState::Closing;
            self.sent
                .write()
                .await
                .push(WebSocketMessage::Close(Some(status)));
            *state = SessionState::Closed;
            Ok(())
        })
    }
}

/// 将一个消息转换为二进制 payload；文本按 UTF-8 编码。
#[must_use]
pub fn payload_bytes(message: &WebSocketMessage) -> Bytes {
    match message {
        WebSocketMessage::Text(text) => Bytes::copy_from_slice(text.as_bytes()),
        WebSocketMessage::Binary(payload)
        | WebSocketMessage::Ping(payload)
        | WebSocketMessage::Pong(payload)
        | WebSocketMessage::Continuation { payload, .. } => payload.clone(),
        WebSocketMessage::Close(Some(status)) => {
            Bytes::copy_from_slice(&status.code().as_u16().to_be_bytes())
        }
        WebSocketMessage::Close(None) => Bytes::new(),
    }
}

/// 将内部关闭码转换为可发送的基础关闭码。
#[must_use]
pub const fn normalize_close_code(code: CloseCode) -> CloseCode {
    if code.is_wire_valid() {
        code
    } else {
        CloseCode::ProtocolError
    }
}
