//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.SockJsFrameType
//!
//! SockJS 帧类型。

/// SockJS 帧类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SockJsFrameType {
    /// 开帧：`o`。
    Open,
    /// 心跳帧：`h`。
    Heartbeat,
    /// 消息帧：`a[...]` 或 `m...`。
    Message,
    /// 关闭帧：`c[code,"reason"]`。
    Close,
}
