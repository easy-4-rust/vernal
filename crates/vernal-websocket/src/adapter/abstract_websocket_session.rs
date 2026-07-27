//! 对应 Java 类：org.springframework.web.socket.adapter.AbstractWebSocketSession
//!
//! 抽象基类，持有 attributes、原生 session（懒初始化）。
//! Spring 中 sendMessage 把消息分派到 `send_text_message/binary/ping/pong` 的
//! protected 抽象方法；Rust 把这一层做薄：用 `native` 引用 + handler 闭包表达。

use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use http::{HeaderMap, Uri};
use tokio::sync::RwLock;

use crate::{
    CloseStatus, HandlerFuture, SessionState, WebSocketError, WebSocketMessage, WebSocketSession,
};

/// 携带 attributes 与 native 引用的抽象 session。
pub struct AbstractWebSocketSession<N: Send + Sync + 'static> {
    id: String,
    uri: Option<Uri>,
    headers: HeaderMap,
    accepted_protocol: Option<String>,
    attributes: RwLock<BTreeMap<String, String>>,
    state: RwLock<SessionState>,
    native: RwLock<Option<Arc<N>>>,
}

impl<N: Send + Sync + 'static> AbstractWebSocketSession<N> {
    /// 创建未初始化 native 的 session。
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        uri: Option<Uri>,
        headers: HeaderMap,
        accepted_protocol: Option<String>,
        attributes: BTreeMap<String, String>,
    ) -> Self {
        Self {
            id: id.into(),
            uri,
            headers,
            accepted_protocol,
            attributes: RwLock::new(attributes),
            state: RwLock::new(SessionState::Open),
            native: RwLock::new(None),
        }
    }

    /// 初始化 native session（Spring `initializeNativeSession`）。
    pub async fn initialize_native_session(&self, native: Arc<N>) {
        *self.native.write().await = Some(native);
    }

    /// 返回 native session 引用（必须已初始化）。
    ///
    /// # Errors
    ///
    /// 当 native 未初始化时返回错误。
    pub async fn native_session(&self) -> Result<Arc<N>, WebSocketError> {
        self.native.read().await.clone().ok_or_else(|| {
            WebSocketError::protocol(
                crate::CloseCode::ServerError,
                "WebSocket session not yet initialized",
            )
        })
    }

    /// 读取 attribute。
    pub async fn attribute(&self, key: &str) -> Option<String> {
        self.attributes.read().await.get(key).cloned()
    }

    /// 写入 attribute。
    pub async fn set_attribute(&self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes
            .write()
            .await
            .insert(key.into(), value.into());
    }
}

impl<N: Send + Sync + 'static> WebSocketSession for AbstractWebSocketSession<N> {
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

    fn send(&self, _message: WebSocketMessage) -> HandlerFuture<'_, Result<(), WebSocketError>> {
        Box::pin(async move {
            self.native_session().await?;
            // 真实 transport 由具体 adapter 重写；这里仅保证 native 已就绪。
            Ok(())
        })
    }

    fn close(&self, _status: CloseStatus) -> HandlerFuture<'_, Result<(), WebSocketError>> {
        Box::pin(async move {
            self.native_session().await?;
            *self.state.write().await = SessionState::Closed;
            Ok(())
        })
    }
}

/// 工具：将控制消息 payload 安全规约（Ping/Pong <=125 字节）。
#[must_use]
pub fn ensure_control_payload(payload: &Bytes) -> Result<(), WebSocketError> {
    if payload.len() > 125 {
        Err(WebSocketError::LimitExceeded {
            kind: "control-payload",
            limit: 125,
            observed: payload.len(),
        })
    } else {
        Ok(())
    }
}
