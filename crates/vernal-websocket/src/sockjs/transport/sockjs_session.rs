//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.SockJsSession
//!
//! SockJS session 扩展 trait。对标 Spring `SockJsSession` 的核心方法：
//! `getTimeSinceLastActive` 与 `disableHeartbeat`。
//! Rust 不要求继承 WebSocketSession，通过组合表达。

use std::time::{Duration, Instant};

/// SockJS session 扩展 trait。
pub trait SockJsSession: Send + Sync {
    /// 返回自上次活跃以来的时间；新 session 返回创建以来的时间。
    fn time_since_last_active_dyn(&self) -> Duration;

    /// 禁用 SockJS 心跳（通常因上层协议已自带心跳）。
    fn disable_heartbeat_dyn(&self);
}

/// SockJS session 共享状态。
#[derive(Debug)]
pub struct SockJsSessionState {
    last_active: std::sync::Mutex<Instant>,
    heartbeat_disabled: std::sync::atomic::AtomicBool,
}

impl SockJsSessionState {
    /// 创建状态，记录当前时间。
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_active: std::sync::Mutex::new(Instant::now()),
            heartbeat_disabled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 标记为活跃。
    pub fn touch(&self) {
        *self.last_active.lock().expect("last_active poisoned") = Instant::now();
    }

    /// 返回距离上次活跃的时间。
    #[must_use]
    pub fn time_since_last_active(&self) -> Duration {
        self.last_active
            .lock()
            .expect("last_active poisoned")
            .elapsed()
    }

    /// 禁用心跳。
    pub fn disable_heartbeat(&self) {
        self.heartbeat_disabled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// 心跳是否已禁用。
    #[must_use]
    pub fn heartbeat_disabled(&self) -> bool {
        self.heartbeat_disabled
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for SockJsSessionState {
    fn default() -> Self {
        Self::new()
    }
}
