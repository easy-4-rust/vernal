//! WebSocket 消息类型。

use bytes::Bytes;

use crate::CloseStatus;

/// WebSocket 消息种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKind {
    /// 文本消息。
    Text,
    /// 二进制消息。
    Binary,
    /// Ping 控制消息。
    Ping,
    /// Pong 控制消息。
    Pong,
    /// 关闭消息。
    Close,
    /// 延续分片。
    Continuation,
}

/// 框架稳定的 WebSocket 消息模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketMessage {
    /// UTF-8 文本消息。
    Text(String),
    /// 二进制消息。
    Binary(Bytes),
    /// Ping 控制消息。
    Ping(Bytes),
    /// Pong 控制消息。
    Pong(Bytes),
    /// 关闭消息。
    Close(Option<CloseStatus>),
    /// 分片消息；默认 handler 只在显式支持部分消息时接收。
    Continuation {
        /// 分片内容。
        payload: Bytes,
        /// 是否为最后一个分片。
        last: bool,
    },
}

impl WebSocketMessage {
    /// 创建文本消息。
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// 创建二进制消息。
    #[must_use]
    pub fn binary(payload: impl Into<Bytes>) -> Self {
        Self::Binary(payload.into())
    }

    /// 返回消息种类。
    #[must_use]
    pub const fn kind(&self) -> MessageKind {
        match self {
            Self::Text(_) => MessageKind::Text,
            Self::Binary(_) => MessageKind::Binary,
            Self::Ping(_) => MessageKind::Ping,
            Self::Pong(_) => MessageKind::Pong,
            Self::Close(_) => MessageKind::Close,
            Self::Continuation { .. } => MessageKind::Continuation,
        }
    }

    /// 返回 payload 字节长度。
    #[must_use]
    pub fn payload_len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Binary(payload)
            | Self::Ping(payload)
            | Self::Pong(payload)
            | Self::Continuation { payload, .. } => payload.len(),
            Self::Close(Some(status)) => status.reason().len().saturating_add(2),
            Self::Close(None) => 0,
        }
    }

    /// 返回该消息是否为控制消息。
    #[must_use]
    pub const fn is_control(&self) -> bool {
        matches!(self, Self::Ping(_) | Self::Pong(_) | Self::Close(_))
    }

    /// 返回该消息是否为最后一个完整消息或分片。
    #[must_use]
    pub const fn is_last(&self) -> bool {
        match self {
            Self::Continuation { last, .. } => *last,
            _ => true,
        }
    }
}
