//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.session.AbstractHttpSockJsSession
//! 以及 PollingSockJsSession / StreamingSockJsSession
//!
//! HTTP 传输 SockJS session：接收 POST 消息、按 transport 格式推送帧。
//! 对标 Spring `AbstractHttpSockJsSession` + `PollingSockJsSession` + `StreamingSockJsSession`。

use std::time::Duration;

use tokio::sync::Mutex;

use crate::sockjs::frame::sockjs_frame::SockJsFrame;
use crate::sockjs::frame::sockjs_frame_format::SockJsFrameFormat;
use crate::sockjs::transport::session::abstract_sockjs_session::AbstractSockJsSession;

/// HTTP SockJS session。合并 AbstractHttpSockJsSession + PollingSockJsSession + StreamingSockJsSession。
pub struct HttpSockJsSession {
    base: AbstractSockJsSession,
    /// 是否正在处理请求（对标 Spring `isActive`）。
    active: Mutex<bool>,
    /// 流式传输已发送字节数（对标 Spring streamBytesLimit）。
    stream_bytes_sent: Mutex<usize>,
    stream_bytes_limit: usize,
}

impl HttpSockJsSession {
    /// 创建 HTTP session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        heartbeat_time: Duration,
        disconnect_delay: Duration,
        stream_bytes_limit: usize,
    ) -> Self {
        Self {
            base: AbstractSockJsSession::new(session_id, heartbeat_time, disconnect_delay),
            active: Mutex::new(false),
            stream_bytes_sent: Mutex::new(0),
            stream_bytes_limit,
        }
    }

    /// 是否有活跃请求。
    pub async fn is_active(&self) -> bool {
        *self.active.lock().await
    }

    /// 处理初次请求：返回 open 帧 + 缓存消息。
    pub async fn handle_initial_request(&self, frame_format: &dyn SockJsFrameFormat) -> Vec<u8> {
        self.base.open().await;
        let open = SockJsFrame::open_frame();
        let formatted = frame_format.format(&open);
        let messages = self.base.drain_messages().await;
        let mut output = formatted.into_bytes();
        if !messages.is_empty() {
            let message_frame = SockJsFrame::message_frame_dummy(&messages);
            let formatted_msg = frame_format.format(&message_frame);
            output.extend_from_slice(formatted_msg.as_bytes());
        }
        *self.active.lock().await = true;
        output
    }

    /// 处理后续请求：返回缓存消息或心跳。
    pub async fn handle_successive_request(&self, frame_format: &dyn SockJsFrameFormat) -> Vec<u8> {
        let messages = self.base.drain_messages().await;
        let frame = if messages.is_empty() {
            SockJsFrame::heartbeat_frame()
        } else {
            SockJsFrame::message_frame_dummy(&messages)
        };
        let formatted = frame_format.format(&frame);
        *self.active.lock().await = true;
        formatted.into_bytes()
    }

    /// 交付客户端发来的消息（对标 Spring `delegateMessages`）。
    pub async fn delegate_messages(&self, messages: &[String]) {
        for message in messages {
            self.base.enqueue_message(message.clone()).await;
        }
        self.base.touch().await;
    }

    /// 返回 stream bytes limit。
    #[must_use]
    pub const fn stream_bytes_limit(&self) -> usize {
        self.stream_bytes_limit
    }

    /// 返回已发送的 stream bytes。
    pub async fn stream_bytes_sent(&self) -> usize {
        *self.stream_bytes_sent.lock().await
    }

    /// 累加 stream bytes（streaming transport 用）。
    pub async fn add_stream_bytes(&self, count: usize) -> bool {
        let mut sent = self.stream_bytes_sent.lock().await;
        *sent += count;
        *sent >= self.stream_bytes_limit
    }

    /// 返回基础 session 引用。
    #[must_use]
    pub fn base(&self) -> &AbstractSockJsSession {
        &self.base
    }
}

/// WebSocketServerSockJsSession：WebSocket 传输的 SockJS session。
/// 对标 Spring `WebSocketServerSockJsSession`。
pub struct WebSocketServerSockJsSession {
    base: AbstractSockJsSession,
}

impl WebSocketServerSockJsSession {
    /// 创建 session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        heartbeat_time: Duration,
        disconnect_delay: Duration,
    ) -> Self {
        Self {
            base: AbstractSockJsSession::new(session_id, heartbeat_time, disconnect_delay),
        }
    }

    /// 返回基础 session 引用。
    #[must_use]
    pub fn base(&self) -> &AbstractSockJsSession {
        &self.base
    }
}

/// SockJsFrame 辅助：用 codec 编码消息。此处用 dummy 避免循环依赖；
/// 实际帧格式在 transport handler 中通过 SockJsServiceConfig 的 codec 完成。
impl SockJsFrame {
    /// 从字符串列表构造 message 帧（不经过 codec，直接拼 JSON 数组）。
    #[must_use]
    pub fn message_frame_dummy(messages: &[String]) -> Self {
        let json_array = messages
            .iter()
            .map(|m| format!("\"{}\"", m.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(",");
        SockJsFrame::new(format!("a[{json_array}]")).unwrap_or_else(|_| SockJsFrame::open_frame())
    }
}
