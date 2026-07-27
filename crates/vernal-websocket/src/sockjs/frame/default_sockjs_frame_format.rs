//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.DefaultSockJsFrameFormat
//!
//! 默认实现：仅在帧内容后追加换行符（XHR/streaming 用）。

use crate::sockjs::frame::sockjs_frame::SockJsFrame;
use crate::sockjs::frame::sockjs_frame_format::SockJsFrameFormat;

/// 默认帧格式：内容 + `\n`。
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSockJsFrameFormat;

impl SockJsFrameFormat for DefaultSockJsFrameFormat {
    fn format(&self, frame: &SockJsFrame) -> String {
        format!("{}\n", frame.content())
    }
}
