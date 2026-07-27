//! 消息 trait。

use std::collections::BTreeMap;

/// 消息 trait。
///
/// 对标 Spring 的 `Message<T>`。
pub trait Message: Send + Sync {
    /// 消息 ID。
    fn id(&self) -> &str;

    /// 消息负载。
    fn payload(&self) -> &[u8];

    /// 消息头（可选实现）。
    fn headers(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }
}

/// 通用字节消息实现。
#[derive(Debug, Clone)]
pub struct GenericMessage {
    /// 消息 ID。
    pub message_id: String,
    /// 负载。
    pub data: Vec<u8>,
    /// 头。
    pub header_map: BTreeMap<String, String>,
}

impl GenericMessage {
    /// 创建消息。
    #[must_use]
    pub fn new(id: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            message_id: id.into(),
            data,
            header_map: BTreeMap::new(),
        }
    }

    /// 添加头。
    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_map.insert(name.into(), value.into());
        self
    }
}

impl Message for GenericMessage {
    fn id(&self) -> &str {
        &self.message_id
    }
    fn payload(&self) -> &[u8] {
        &self.data
    }
    fn headers(&self) -> BTreeMap<String, String> {
        self.header_map.clone()
    }
}
