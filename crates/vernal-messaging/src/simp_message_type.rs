//! SIMP 消息类型。对标 Spring `org.springframework.messaging.simp.SimpMessageType`。

/// SIMP 消息类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpMessageType {
    /// CONNECT/STOMP。
    Connect,
    /// DISCONNECT。
    Disconnect,
    /// SUBSCRIBE。
    Subscribe,
    /// UNSUBSCRIBE。
    Unsubscribe,
    /// SEND/MESSAGE。
    Message,
    /// HEARTBEAT。
    Heartbeat,
    /// 其它。
    Other,
}

impl SimpMessageType {
    /// 返回 Spring header 字符串值。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connect => "CONNECT",
            Self::Disconnect => "DISCONNECT",
            Self::Subscribe => "SUBSCRIBE",
            Self::Unsubscribe => "UNSUBSCRIBE",
            Self::Message => "MESSAGE",
            Self::Heartbeat => "HEARTBEAT",
            Self::Other => "OTHER",
        }
    }
}
