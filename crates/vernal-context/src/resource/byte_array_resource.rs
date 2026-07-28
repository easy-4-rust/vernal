//! 字节数组资源。
//!
//! 对标 Spring `org.springframework.core.io.ByteArrayResource`。

use super::resource::Resource;
use std::io;

/// 字节数组资源。
///
/// 对应 Java: org.springframework.core.io.ByteArrayResource
#[derive(Debug, Clone)]
pub struct ByteArrayResource {
    bytes: Vec<u8>,
    description: String,
}

impl ByteArrayResource {
    /// 从字节数组创建资源。
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            description: "字节数组资源".to_string(),
        }
    }

    /// 从字节数组与描述创建资源。
    #[must_use]
    pub fn with_description(bytes: impl Into<Vec<u8>>, description: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            description: description.into(),
        }
    }
}

impl Resource for ByteArrayResource {
    fn exists(&self) -> bool {
        true
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn filename(&self) -> Option<&str> {
        None
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}
