//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.SockJsServiceConfig
//!
//! SockJS 服务配置：stream bytes limit / heartbeat time / http message cache / codec。
//! Spring 用 `TaskScheduler`；Vernal 不绑定调度器实现，仅暴露配置。

use std::sync::Arc;
use std::time::Duration;

use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;

/// SockJS 服务配置。
#[derive(Clone)]
pub struct SockJsServiceConfig {
    /// 流式传输在回收连接前发送的最小字节数（默认 128 KiB）。
    pub stream_bytes_limit: usize,
    /// 心跳间隔（默认 25 秒）。
    pub heartbeat_time: Duration,
    /// HTTP 消息缓存数量（默认 100）。
    pub http_message_cache_size: usize,
    /// 消息 codec。
    pub message_codec: Arc<dyn SockJsMessageCodec>,
}

impl std::fmt::Debug for SockJsServiceConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SockJsServiceConfig")
            .field("stream_bytes_limit", &self.stream_bytes_limit)
            .field("heartbeat_time", &self.heartbeat_time)
            .field("http_message_cache_size", &self.http_message_cache_size)
            .finish_non_exhaustive()
    }
}

impl Default for SockJsServiceConfig {
    fn default() -> Self {
        Self {
            stream_bytes_limit: 128 * 1024,
            heartbeat_time: Duration::from_millis(25_000),
            http_message_cache_size: 100,
            message_codec: Arc::new(
                crate::sockjs::frame::json_sockjs_message_codec::JsonSockJsMessageCodec::new(),
            ),
        }
    }
}

impl SockJsServiceConfig {
    /// 创建默认配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
