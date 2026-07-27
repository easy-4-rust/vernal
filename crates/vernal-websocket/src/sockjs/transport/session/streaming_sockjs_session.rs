//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.session.StreamingSockJsSession
//!
//! 流式 HTTP 传输的 SockJS session。
//! 行为差异（对比 PollingSockJsSession）：
//! 1. initial 请求先写 prelude（2KB 空格填充），再写 open 帧，然后 flushCache；
//! 2. successive 请求也写 prelude 再 flushCache；
//! 3. flushCache 逐条消息写帧，累计 byteCount，达到 streamBytesLimit 时回收请求（结束 HTTP 响应）。

use std::sync::Arc;
use std::time::Duration;

use crate::sockjs::frame::sockjs_frame::SockJsFrame;
use crate::sockjs::frame::sockjs_frame_format::SockJsFrameFormat;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use crate::sockjs::transport::session::abstract_sockjs_session::{
    AbstractSockJsSession, SessionLifecycle,
};

/// StreamingSockJsSession。对标 Spring `StreamingSockJsSession`。
///
/// 与 PollingSockJsSession 的核心差异：
/// 1. 有 prelude（2KB padding）；
/// 2. flushCache 逐条消息写帧，按 byteCount 累计，达到 streamBytesLimit 时回收；
/// 3. writeFrame 不调用 resetRequest（保持 HTTP 流不关闭）。
pub struct StreamingSockJsSession {
    base: AbstractSockJsSession,
    codec: Arc<dyn SockJsMessageCodec>,
    stream_bytes_limit: usize,
    byte_count: tokio::sync::Mutex<usize>,
}

/// Spring prelude 大小（2048 字节填充）。
const PRELUDE_SIZE: usize = 2048;

impl StreamingSockJsSession {
    /// 创建 streaming session。
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        heartbeat_time: Duration,
        disconnect_delay: Duration,
        stream_bytes_limit: usize,
        codec: Arc<dyn SockJsMessageCodec>,
    ) -> Self {
        Self {
            base: AbstractSockJsSession::new(session_id, heartbeat_time, disconnect_delay),
            codec,
            stream_bytes_limit,
            byte_count: tokio::sync::Mutex::new(0),
        }
    }

    /// 处理 initial/successive 请求（对标 Spring `handleRequestInternal`）。
    /// 返回写入 HTTP 响应的字节。
    pub async fn handle_request(
        &self,
        initial: bool,
        frame_format: &dyn SockJsFrameFormat,
    ) -> Vec<u8> {
        let mut output = Vec::new();

        // Prelude：2KB 空格填充（对标 Spring `writePrelude`）
        output.resize(PRELUDE_SIZE, b' ');
        output.push(b'\n');

        if initial {
            self.base.open().await;
            let open_frame = SockJsFrame::open_frame();
            output.extend_from_slice(frame_format.format(&open_frame).as_bytes());
        }

        // flushCache：逐条消息写帧，累计 byteCount
        let mut byte_count = self.byte_count.lock().await;
        let messages = self.base.drain_messages().await;
        for message in &messages {
            let frame = SockJsFrame::message_frame(self.codec.as_ref(), &[message.as_str()]);
            let formatted = frame_format.format(&frame);
            let frame_bytes = formatted.as_bytes();
            *byte_count += frame_bytes.len() + 1; // +1 for newline
            output.extend_from_slice(frame_bytes);

            if *byte_count >= self.stream_bytes_limit {
                // 回收当前请求（对标 Spring `resetRequest` + `byteCount = 0`）
                *byte_count = 0;
                break;
            }
        }

        // 如果没有消息，写 heartbeat
        if messages.is_empty() {
            let heartbeat = SockJsFrame::heartbeat_frame();
            output.extend_from_slice(frame_format.format(&heartbeat).as_bytes());
        }

        output
    }

    /// 返回当前 byteCount（用于测试验证回收逻辑）。
    pub async fn byte_count(&self) -> usize {
        *self.byte_count.lock().await
    }

    /// 返回基础 session 引用。
    #[must_use]
    pub fn base(&self) -> &AbstractSockJsSession {
        &self.base
    }
}
