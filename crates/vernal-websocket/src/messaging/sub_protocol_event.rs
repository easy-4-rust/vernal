//! 对应 Java 类：org.springframework.web.socket.messaging.AbstractSubProtocolEvent 及其子类
//!
//! STOMP/SIMP 子协议生命周期事件。对标 Spring 5 个事件类。

use std::collections::BTreeMap;

/// 子协议事件基类载荷。对标 Spring `AbstractSubProtocolEvent`。
#[derive(Debug, Clone)]
pub struct SubProtocolEvent {
    /// 关联 source（通常是 handler 名称）。
    pub source: String,
    /// session id。
    pub session_id: String,
    /// user（可能为 None）。
    pub user: Option<String>,
    /// 事件特定 headers。
    pub headers: BTreeMap<String, String>,
}

impl SubProtocolEvent {
    /// 创建事件。
    #[must_use]
    pub fn new(source: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            session_id: session_id.into(),
            user: None,
            headers: BTreeMap::new(),
        }
    }

    /// 设置 user。
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// CONNECT 事件。对标 `SessionConnectEvent`。
pub type SessionConnectEvent = SubProtocolEvent;
/// CONNECTED 事件。对标 `SessionConnectedEvent`。
pub type SessionConnectedEvent = SubProtocolEvent;
/// DISCONNECT 事件。对标 `SessionDisconnectEvent`。
#[derive(Debug, Clone)]
pub struct SessionDisconnectEvent {
    /// 基础事件。
    pub event: SubProtocolEvent,
    /// 关闭码。
    pub close_code: u16,
    /// 关闭原因。
    pub close_reason: String,
}

impl SessionDisconnectEvent {
    /// 创建 DISCONNECT 事件。
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        session_id: impl Into<String>,
        close_code: u16,
        close_reason: impl Into<String>,
    ) -> Self {
        Self {
            event: SubProtocolEvent::new(source, session_id),
            close_code,
            close_reason: close_reason.into(),
        }
    }
}

/// SUBSCRIBE 事件。对标 `SessionSubscribeEvent`。
#[derive(Debug, Clone)]
pub struct SessionSubscribeEvent {
    /// 基础事件。
    pub event: SubProtocolEvent,
    /// 订阅 id。
    pub subscription_id: String,
    /// 订阅 destination。
    pub destination: String,
}

impl SessionSubscribeEvent {
    /// 创建 SUBSCRIBE 事件。
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        session_id: impl Into<String>,
        subscription_id: impl Into<String>,
        destination: impl Into<String>,
    ) -> Self {
        Self {
            event: SubProtocolEvent::new(source, session_id),
            subscription_id: subscription_id.into(),
            destination: destination.into(),
        }
    }
}

/// UNSUBSCRIBE 事件。对标 `SessionUnsubscribeEvent`。
#[derive(Debug, Clone)]
pub struct SessionUnsubscribeEvent {
    /// 基础事件。
    pub event: SubProtocolEvent,
    /// 订阅 id。
    pub subscription_id: String,
}

impl SessionUnsubscribeEvent {
    /// 创建 UNSUBSCRIBE 事件。
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        session_id: impl Into<String>,
        subscription_id: impl Into<String>,
    ) -> Self {
        Self {
            event: SubProtocolEvent::new(source, session_id),
            subscription_id: subscription_id.into(),
        }
    }
}
