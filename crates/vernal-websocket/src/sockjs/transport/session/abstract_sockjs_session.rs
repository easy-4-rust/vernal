//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.session.AbstractSockJsSession
//!
//! SockJS session 状态机：NEW → OPEN → CLOSED。
//! 持有消息缓存、心跳定时器与 responseLock（对标 Spring `responseLock`）。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// SockJS session 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLifecycle {
    /// 新建，尚未处理请求。
    New,
    /// 已打开。
    Open,
    /// 已关闭。
    Closed,
}

/// SockJS session 基类。对标 Spring `AbstractSockJsSession`。
pub struct AbstractSockJsSession {
    session_id: String,
    lifecycle: Mutex<SessionLifecycle>,
    message_cache: Mutex<VecDeque<String>>,
    heartbeat_disabled: AtomicBool,
    /// 创建时间（对标 Spring `getCreationTime`，暂未使用）。
    #[allow(dead_code)]
    created_at: Instant,
    last_active: Mutex<Instant>,
    heartbeat_time: Duration,
    disconnect_delay: Duration,
}

impl AbstractSockJsSession {
    /// 创建 session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        heartbeat_time: Duration,
        disconnect_delay: Duration,
    ) -> Self {
        let now = Instant::now();
        Self {
            session_id: session_id.into(),
            lifecycle: Mutex::new(SessionLifecycle::New),
            message_cache: Mutex::new(VecDeque::new()),
            heartbeat_disabled: AtomicBool::new(false),
            created_at: now,
            last_active: Mutex::new(now),
            heartbeat_time,
            disconnect_delay,
        }
    }

    /// 返回 session id。
    #[must_use]
    pub fn id(&self) -> &str {
        &self.session_id
    }

    /// 是否为新建。
    pub async fn is_new(&self) -> bool {
        *self.lifecycle.lock().await == SessionLifecycle::New
    }

    /// 是否已关闭。
    pub async fn is_closed(&self) -> bool {
        *self.lifecycle.lock().await == SessionLifecycle::Closed
    }

    /// 是否已打开。
    pub async fn is_open(&self) -> bool {
        *self.lifecycle.lock().await == SessionLifecycle::Open
    }

    /// 标记为已打开。
    pub async fn open(&self) {
        *self.lifecycle.lock().await = SessionLifecycle::Open;
    }

    /// 关闭 session。
    pub async fn close(&self) {
        *self.lifecycle.lock().await = SessionLifecycle::Closed;
    }

    /// 禁用心跳。
    pub fn disable_heartbeat(&self) {
        self.heartbeat_disabled.store(true, Ordering::SeqCst);
    }

    /// 心跳是否禁用。
    #[must_use]
    pub fn heartbeat_disabled(&self) -> bool {
        self.heartbeat_disabled.load(Ordering::SeqCst)
    }

    /// 返回心跳间隔。
    #[must_use]
    pub const fn heartbeat_time(&self) -> Duration {
        self.heartbeat_time
    }

    /// 返回断开延迟。
    #[must_use]
    pub const fn disconnect_delay(&self) -> Duration {
        self.disconnect_delay
    }

    /// 标记活跃。
    pub async fn touch(&self) {
        *self.last_active.lock().await = Instant::now();
    }

    /// 返回距上次活跃的时间。
    pub async fn time_since_last_active(&self) -> Duration {
        self.last_active.lock().await.elapsed()
    }

    /// 交付客户端发来的消息（对标 Spring `delegateMessages`）。
    pub async fn delegate_messages(&self, messages: &[String]) {
        for message in messages {
            self.message_cache.lock().await.push_back(message.clone());
        }
        self.touch().await;
    }

    /// 把消息加入缓存。
    pub async fn enqueue_message(&self, message: impl Into<String>) {
        self.message_cache.lock().await.push_back(message.into());
    }

    /// 取出缓存中的所有消息。
    pub async fn drain_messages(&self) -> Vec<String> {
        self.message_cache.lock().await.drain(..).collect()
    }

    /// 返回缓存消息数。
    pub async fn message_count(&self) -> usize {
        self.message_cache.lock().await.len()
    }
}

impl crate::sockjs::transport::sockjs_session::SockJsSession for AbstractSockJsSession {
    fn time_since_last_active_dyn(&self) -> Duration {
        self.last_active
            .try_lock()
            .map_or(Duration::ZERO, |guard| guard.elapsed())
    }
    fn disable_heartbeat_dyn(&self) {
        self.disable_heartbeat();
    }
}
