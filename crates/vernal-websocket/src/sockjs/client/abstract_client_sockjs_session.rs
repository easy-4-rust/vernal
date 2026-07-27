//! 对应 Java 类：org.springframework.web.socket.sockjs.client.AbstractClientSockJsSession
//! 以及 XhrClientSockJsSession / WebSocketClientSockJsSession
//!
//! 客户端 SockJS session：管理消息缓存、生命周期回调、关闭。
//! 对标 Spring `AbstractClientSockJsSession` + `XhrClientSockJsSession` + `WebSocketClientSockJsSession`。

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::WebSocketHandler;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;

/// 客户端 session 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSessionState {
    /// 新建。
    New,
    /// 已打开。
    Open,
    /// 已关闭。
    Closed,
}

/// 客户端 SockJS session。合并 Spring `AbstractClientSockJsSession`
/// + `XhrClientSockJsSession` + `WebSocketClientSockJsSession`。
pub struct ClientSockJsSession {
    session_id: String,
    state: Mutex<ClientSessionState>,
    message_cache: Mutex<VecDeque<String>>,
    codec: Arc<dyn SockJsMessageCodec>,
    handler: Arc<dyn WebSocketHandler>,
    heartbeat_disabled: AtomicBool,
    created_at: Instant,
    last_active: Mutex<Instant>,
}

impl ClientSockJsSession {
    /// 创建客户端 session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        codec: Arc<dyn SockJsMessageCodec>,
        handler: Arc<dyn WebSocketHandler>,
    ) -> Self {
        let now = Instant::now();
        Self {
            session_id: session_id.into(),
            state: Mutex::new(ClientSessionState::New),
            message_cache: Mutex::new(VecDeque::new()),
            codec,
            handler,
            heartbeat_disabled: AtomicBool::new(false),
            created_at: now,
            last_active: Mutex::new(now),
        }
    }

    /// 返回 session id。
    #[must_use]
    pub fn id(&self) -> &str {
        &self.session_id
    }

    /// 返回当前状态。
    pub async fn state(&self) -> ClientSessionState {
        *self.state.lock().await
    }

    /// 是否已打开。
    pub async fn is_open(&self) -> bool {
        *self.state.lock().await == ClientSessionState::Open
    }

    /// 是否已关闭。
    pub async fn is_closed(&self) -> bool {
        *self.state.lock().await == ClientSessionState::Closed
    }

    /// 打开 session（收到 open 帧时调用）。
    pub async fn open(&self) {
        *self.state.lock().await = ClientSessionState::Open;
    }

    /// 关闭 session。
    pub async fn close(&self) {
        *self.state.lock().await = ClientSessionState::Closed;
    }

    /// 禁用心跳。
    pub fn disable_heartbeat(&self) {
        self.heartbeat_disabled.store(true, Ordering::SeqCst);
    }

    /// 心跳是否已禁用。
    #[must_use]
    pub fn heartbeat_disabled(&self) -> bool {
        self.heartbeat_disabled.load(Ordering::SeqCst)
    }

    /// 把 SockJS 消息帧内容解码后交付 handler。
    pub async fn handle_frame(&self, frame_content: &str) {
        self.touch().await;
        if frame_content == "o" {
            self.open().await;
            return;
        }
        if frame_content == "h" {
            return;
        }
        if let Some(rest) = frame_content.strip_prefix('a') {
            if let Ok(messages) = self.codec.decode(rest) {
                for message in messages {
                    self.message_cache.lock().await.push_back(message);
                }
            }
            return;
        }
        if frame_content.starts_with('c') {
            self.close().await;
        }
    }

    /// 发送消息（对标 Spring `sendMessage`：编码为 SockJS message 帧）。
    pub fn encode_outbound(&self, messages: &[&str]) -> String {
        self.codec.encode(messages)
    }

    /// 取出收到的消息。
    pub async fn drain_messages(&self) -> Vec<String> {
        self.message_cache.lock().await.drain(..).collect()
    }

    /// 返回缓存消息数。
    pub async fn message_count(&self) -> usize {
        self.message_cache.lock().await.len()
    }

    async fn touch(&self) {
        *self.last_active.lock().await = Instant::now();
    }

    /// 返回距上次活跃的时间。
    pub async fn time_since_last_active(&self) -> Duration {
        self.last_active.lock().await.elapsed()
    }

    /// 返回关联 handler。
    #[must_use]
    pub fn handler(&self) -> &Arc<dyn WebSocketHandler> {
        &self.handler
    }
}

impl std::fmt::Debug for ClientSockJsSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientSockJsSession")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}
