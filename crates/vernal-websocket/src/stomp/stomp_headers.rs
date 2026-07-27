//! 对应 Java 类：org.springframework.messaging.simp.stomp.StompHeaders
//!
//! STOMP 帧头。对标 Spring `StompHeaders` 的常用 header 常量与访问方法。

use std::collections::BTreeMap;

/// STOMP header 常量。
pub mod headers {
    /// accept-version
    pub const ACCEPT_VERSION: &str = "accept-version";
    /// host
    pub const HOST: &str = "host";
    /// version
    pub const VERSION: &str = "version";
    /// login
    pub const LOGIN: &str = "login";
    /// passcode
    pub const PASSCODE: &str = "passcode";
    /// heart-beat
    pub const HEART_BEAT: &str = "heart-beat";
    /// destination
    pub const DESTINATION: &str = "destination";
    /// id
    pub const ID: &str = "id";
    /// subscription
    pub const SUBSCRIPTION: &str = "subscription";
    /// message-id
    pub const MESSAGE_ID: &str = "message-id";
    /// message (ERROR 帧的简短描述)
    pub const MESSAGE: &str = "message";
    /// receipt
    pub const RECEIPT: &str = "receipt";
    /// receipt-id
    pub const RECEIPT_ID: &str = "receipt-id";
    /// ack
    pub const ACK: &str = "ack";
    /// transaction
    pub const TRANSACTION: &str = "transaction";
    /// content-length
    pub const CONTENT_LENGTH: &str = "content-length";
    /// content-type
    pub const CONTENT_TYPE: &str = "content-type";
}

/// STOMP 帧头集合（按出现顺序保留，重复 header 取首个值）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StompHeaders {
    headers: BTreeMap<String, String>,
}

impl StompHeaders {
    /// 创建空集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 header（覆盖）。
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(name.into(), value.into());
    }

    /// 添加 header（保留首个值，对标 STOMP 协议 first-wins）。
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        self.headers.entry(name).or_insert(value.into());
    }

    /// 读取 header。
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }

    /// 移除 header。
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.remove(name)
    }

    /// 返回所有 header 的引用。
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.headers.iter()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// 返回 header 数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// destination。
    #[must_use]
    pub fn destination(&self) -> Option<&str> {
        self.get(headers::DESTINATION)
    }

    /// subscription id。
    #[must_use]
    pub fn subscription(&self) -> Option<&str> {
        self.get(headers::SUBSCRIPTION)
    }

    /// receipt id。
    #[must_use]
    pub fn receipt(&self) -> Option<&str> {
        self.get(headers::RECEIPT)
    }

    /// content-length 解析为 usize。
    #[must_use]
    pub fn content_length(&self) -> Option<usize> {
        self.get(headers::CONTENT_LENGTH)
            .and_then(|value| value.parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_preserves_first_value() {
        let mut headers = StompHeaders::new();
        headers.add("id", "first");
        headers.add("id", "second");
        assert_eq!(headers.get("id"), Some("first"));
    }

    #[test]
    fn set_overwrites_value() {
        let mut headers = StompHeaders::new();
        headers.set("id", "first");
        headers.set("id", "second");
        assert_eq!(headers.get("id"), Some("second"));
    }

    #[test]
    fn content_length_parses_usize() {
        let mut headers = StompHeaders::new();
        headers.set(headers::CONTENT_LENGTH, "42");
        assert_eq!(headers.content_length(), Some(42));
    }
}
