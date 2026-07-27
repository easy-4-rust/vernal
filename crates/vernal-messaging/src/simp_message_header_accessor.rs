//! SIMP 消息头访问器。对标 Spring `SimpMessageHeaderAccessor` 与 `SimpAttributes`。

use std::collections::BTreeMap;

/// SIMP 消息头常量。
pub mod simp_headers {
    /// simpMessageType
    pub const SIMP_MESSAGE_TYPE: &str = "simpMessageType";
    /// simpSessionId
    pub const SIMP_SESSION_ID: &str = "simpSessionId";
    /// simpSubscriptionId
    pub const SIMP_SUBSCRIPTION_ID: &str = "simpSubscriptionId";
    /// simpDestination
    pub const SIMP_DESTINATION: &str = "simpDestination";
    /// simpUser
    pub const SIMP_USER: &str = "simpUser";
    /// simpConnectMessage
    pub const SIMP_CONNECT_MESSAGE: &str = "simpConnectMessage";
    /// simpDisconnectMessage
    pub const SIMP_DISCONNECT_MESSAGE: &str = "simpDisconnectMessage";
    /// simpHeartbeat
    pub const SIMP_HEARTBEAT: &str = "simpHeartbeat";
    /// simpOrigDestination
    pub const SIMP_ORIG_DESTINATION: &str = "simpOrigDestination";
    /// simpIgnoreError
    pub const SIMP_IGNORE_ERROR: &str = "simpIgnoreError";
    /// simpSessionAttributes
    pub const SIMP_SESSION_ATTRIBUTES: &str = "simpSessionAttributes";
}

/// SIMP 消息头访问器。
#[derive(Debug, Clone, Default)]
pub struct SimpMessageHeaderAccessor {
    headers: BTreeMap<String, String>,
}

impl SimpMessageHeaderAccessor {
    /// 创建空访问器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 header map 创建。
    #[must_use]
    pub fn from_headers(headers: BTreeMap<String, String>) -> Self {
        Self { headers }
    }

    /// 设置 header。
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(name.into(), value.into());
    }

    /// 读取 header。
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }

    /// 返回所有 header。
    #[must_use]
    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    /// session id。
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.get(simp_headers::SIMP_SESSION_ID)
    }

    /// destination。
    #[must_use]
    pub fn destination(&self) -> Option<&str> {
        self.get(simp_headers::SIMP_DESTINATION)
    }

    /// subscription id。
    #[must_use]
    pub fn subscription_id(&self) -> Option<&str> {
        self.get(simp_headers::SIMP_SUBSCRIPTION_ID)
    }

    /// user。
    #[must_use]
    pub fn user(&self) -> Option<&str> {
        self.get(simp_headers::SIMP_USER)
    }
}
