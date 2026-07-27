//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.session.PollingSockJsSession
//!
//! 轮询 HTTP 传输的 SockJS session。
//! 行为：initial → open 帧；有消息 → flushCache（全部消息写入单个帧）；无消息 → heartbeat。
//! writeFrame 后立即 resetRequest（结束 HTTP 响应）。

use std::sync::Arc;
use std::time::Duration;

use crate::sockjs::frame::default_sockjs_frame_format::DefaultSockJsFrameFormat;
use crate::sockjs::frame::sockjs_frame::SockJsFrame;
use crate::sockjs::frame::sockjs_frame_format::SockJsFrameFormat;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use crate::sockjs::transport::session::abstract_sockjs_session::{
    AbstractSockJsSession, SessionLifecycle,
};

/// PollingSockJsSession。对标 Spring `PollingSockJsSession`。
///
/// 与 StreamingSockJsSession 的核心差异：
/// 1. flushCache 把全部缓存消息合并到单个 message 帧（而非逐条发送）；
/// 2. writeFrame 后立即 resetRequest（HTTP 响应结束），不做 stream bytes 回收。
pub struct PollingSockJsSession {
    base: AbstractSockJsSession,
    /// 关联的 message codec（用于 flushCache 编码）。
    codec: Arc<dyn SockJsMessageCodec>,
}

impl PollingSockJsSession {
    /// 创建 polling session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        heartbeat_time: Duration,
        disconnect_delay: Duration,
        codec: Arc<dyn SockJsMessageCodec>,
    ) -> Self {
        Self {
            base: AbstractSockJsSession::new(session_id, heartbeat_time, disconnect_delay),
            codec,
        }
    }

    /// 处理 initial/successive 请求（对标 Spring `handleRequestInternal`）。
    /// 返回写入 HTTP 响应的字节。
    pub async fn handle_request(
        &self,
        initial: bool,
        frame_format: &dyn SockJsFrameFormat,
    ) -> Vec<u8> {
        if initial {
            self.base.open().await;
            let frame = SockJsFrame::open_frame();
            return frame_format.format(&frame).into_bytes();
        }
        let messages = self.base.drain_messages().await;
        if !messages.is_empty() {
            // Polling：全部消息合并为单个 message 帧
            let refs: Vec<&str> = messages.iter().map(String::as_str).collect();
            let frame = SockJsFrame::message_frame(self.codec.as_ref(), &refs);
            return frame_format.format(&frame).into_bytes();
        }
        // 无消息：heartbeat
        let frame = SockJsFrame::heartbeat_frame();
        frame_format.format(&frame).into_bytes()
    }

    /// 返回基础 session 引用。
    #[must_use]
    pub fn base(&self) -> &AbstractSockJsSession {
        &self.base
    }
}
