//! WebSocket 运行配置。

use std::time::Duration;

use crate::BackpressurePolicy;

/// WebSocket 安全与资源配置。
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// 最大消息大小。
    pub max_message_size: usize,
    /// 最大 frame 大小。
    pub max_frame_size: usize,
    /// 出站队列容量。
    pub outbound_capacity: usize,
    /// 入队超时。
    pub enqueue_timeout: Duration,
    /// 背压策略。
    pub backpressure_policy: BackpressurePolicy,
    /// 空闲连接超时。
    pub idle_timeout: Option<Duration>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024,
            max_frame_size: 64 * 1024,
            outbound_capacity: 64,
            enqueue_timeout: Duration::from_secs(5),
            backpressure_policy: BackpressurePolicy::Wait,
            idle_timeout: Some(Duration::from_secs(60)),
        }
    }
}

impl WebSocketConfig {
    /// 创建默认配置构建器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大消息大小。
    #[must_use]
    pub const fn max_message_size(mut self, limit: usize) -> Self {
        self.max_message_size = limit;
        self
    }

    /// 设置最大 frame 大小。
    #[must_use]
    pub const fn max_frame_size(mut self, limit: usize) -> Self {
        self.max_frame_size = limit;
        self
    }

    /// 设置出站队列容量。
    #[must_use]
    pub const fn outbound_capacity(mut self, capacity: usize) -> Self {
        self.outbound_capacity = capacity;
        self
    }

    /// 设置背压策略。
    #[must_use]
    pub const fn backpressure_policy(mut self, policy: BackpressurePolicy) -> Self {
        self.backpressure_policy = policy;
        self
    }
}
