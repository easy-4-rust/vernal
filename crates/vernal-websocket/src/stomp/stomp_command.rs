//! 对应 Java 类：org.springframework.messaging.simp.stomp.StompCommand
//!
//! STOMP 命令。完整复刻 Spring 13 个命令及其元数据（destination/subscriptionId/body）。

/// STOMP 消息类型（对应 Spring `SimpMessageType`）。
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
    /// 其它（ACK/NACK/CONNECTED/RECEIPT/ERROR/BEGIN/COMMIT/ABORT）。
    Other,
}

/// STOMP 命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StompCommand {
    // client → server
    /// STOMP 1.2 握手。
    Stomp,
    /// CONNECT（兼容 1.0/1.1）。
    Connect,
    /// DISCONNECT。
    Disconnect,
    /// SUBSCRIBE。
    Subscribe,
    /// UNSUBSCRIBE。
    Unsubscribe,
    /// SEND。
    Send,
    /// ACK。
    Ack,
    /// NACK。
    Nack,
    /// BEGIN。
    Begin,
    /// COMMIT。
    Commit,
    /// ABORT。
    Abort,
    // server → client
    /// CONNECTED。
    Connected,
    /// RECEIPT。
    Receipt,
    /// MESSAGE。
    Message,
    /// ERROR。
    Error,
}

impl StompCommand {
    /// 返回 STOMP wire 字符串。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stomp => "STOMP",
            Self::Connect => "CONNECT",
            Self::Disconnect => "DISCONNECT",
            Self::Subscribe => "SUBSCRIBE",
            Self::Unsubscribe => "UNSUBSCRIBE",
            Self::Send => "SEND",
            Self::Ack => "ACK",
            Self::Nack => "NACK",
            Self::Begin => "BEGIN",
            Self::Commit => "COMMIT",
            Self::Abort => "ABORT",
            Self::Connected => "CONNECTED",
            Self::Receipt => "RECEIPT",
            Self::Message => "MESSAGE",
            Self::Error => "ERROR",
        }
    }

    /// 从 wire 字符串解析。
    #[must_use]
    pub fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "STOMP" => Self::Stomp,
            "CONNECT" => Self::Connect,
            "DISCONNECT" => Self::Disconnect,
            "SUBSCRIBE" => Self::Subscribe,
            "UNSUBSCRIBE" => Self::Unsubscribe,
            "SEND" => Self::Send,
            "ACK" => Self::Ack,
            "NACK" => Self::Nack,
            "BEGIN" => Self::Begin,
            "COMMIT" => Self::Commit,
            "ABORT" => Self::Abort,
            "CONNECTED" => Self::Connected,
            "RECEIPT" => Self::Receipt,
            "MESSAGE" => Self::Message,
            "ERROR" => Self::Error,
            _ => return None,
        })
    }

    /// 返回对应的 SIMP 消息类型。
    #[must_use]
    pub const fn message_type(self) -> SimpMessageType {
        match self {
            Self::Stomp | Self::Connect => SimpMessageType::Connect,
            Self::Disconnect => SimpMessageType::Disconnect,
            Self::Subscribe => SimpMessageType::Subscribe,
            Self::Unsubscribe => SimpMessageType::Unsubscribe,
            Self::Send | Self::Message => SimpMessageType::Message,
            _ => SimpMessageType::Other,
        }
    }

    /// 是否要求 destination 头。
    #[must_use]
    pub const fn requires_destination(self) -> bool {
        matches!(self, Self::Subscribe | Self::Send | Self::Message)
    }

    /// 是否要求 subscription id 头。
    #[must_use]
    pub const fn requires_subscription_id(self) -> bool {
        matches!(self, Self::Subscribe | Self::Message)
    }

    /// 是否要求 content-length 头。
    #[must_use]
    pub const fn requires_content_length(self) -> bool {
        matches!(self, Self::Send | Self::Message | Self::Error)
    }

    /// 是否允许 body。
    #[must_use]
    pub const fn is_body_allowed(self) -> bool {
        self.requires_content_length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_all_commands() {
        for variant in [
            StompCommand::Stomp,
            StompCommand::Connect,
            StompCommand::Disconnect,
            StompCommand::Subscribe,
            StompCommand::Unsubscribe,
            StompCommand::Send,
            StompCommand::Ack,
            StompCommand::Nack,
            StompCommand::Begin,
            StompCommand::Commit,
            StompCommand::Abort,
            StompCommand::Connected,
            StompCommand::Receipt,
            StompCommand::Message,
            StompCommand::Error,
        ] {
            assert_eq!(StompCommand::from_str(variant.as_str()), Some(variant));
        }
    }

    #[test]
    fn required_headers_match_spring_definitions() {
        assert!(StompCommand::Subscribe.requires_destination());
        assert!(StompCommand::Subscribe.requires_subscription_id());
        assert!(StompCommand::Send.requires_destination());
        assert!(StompCommand::Send.requires_content_length());
        assert!(!StompCommand::Connect.requires_destination());
    }
}
