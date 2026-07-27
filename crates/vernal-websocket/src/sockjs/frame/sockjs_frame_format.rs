//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.SockJsFrameFormat
//!
//! 给 SockJS 帧应用 transport 特定的格式（XHR 追加换行、JSONP 包装等）。

use crate::sockjs::frame::sockjs_frame::SockJsFrame;

/// 帧格式化 SPI。
pub trait SockJsFrameFormat: Send + Sync {
    /// 把帧内容格式化为可写出的字符串。
    fn format(&self, frame: &SockJsFrame) -> String;
}
