//! 编码资源。
//!
//! 对标 Spring `org.springframework.core.io.support.EncodedResource`。

use std::io;

use crate::io::Resource;

/// 编码资源。
///
/// 对应 Java: org.springframework.core.io.support.EncodedResource
///
/// Spring 语义：`Resource + Charset` 组合，`getReader()` 按字符集读取；
/// Rust 中仅支持 UTF-8 读取（对标 JVM 默认字符集语义），描述包含字符集。
pub struct EncodedResource {
    resource: Box<dyn Resource>,
    encoding: String,
}

impl EncodedResource {
    /// 以 UTF-8 编码包装资源。
    #[must_use]
    pub fn new(resource: Box<dyn Resource>) -> Self {
        Self {
            resource,
            encoding: "UTF-8".to_string(),
        }
    }

    /// 返回底层资源。
    #[must_use]
    pub fn resource(&self) -> &dyn Resource {
        self.resource.as_ref()
    }

    /// 返回编码名。
    #[must_use]
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// 按编码读取文本。
    ///
    /// # 错误
    ///
    /// 读取或解码失败时返回 [`std::io::Error`]。
    pub fn read_text(&self) -> io::Result<String> {
        let bytes = self.resource.read_bytes()?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// 生成诊断描述（对标 Spring `getDescription`）。
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "EncodedResource [{}] with encoding [{}]",
            self.resource.description(),
            self.encoding
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn reads_utf8_text() {
        // A 类（合同对齐）：对标 Spring `getReader()` 读取
        let resource = ByteArrayResource::new("你好".as_bytes().to_vec());
        let encoded = EncodedResource::new(Box::new(resource));
        assert_eq!(encoded.read_text().unwrap(), "你好");
        assert_eq!(encoded.encoding(), "UTF-8");
    }

    #[test]
    fn description_includes_encoding() {
        // B 类（边界行为）：对标 Spring 描述格式
        let resource = crate::io::PathResource::new("/tmp/text.txt");
        let encoded = EncodedResource::new(Box::new(resource));
        let desc = encoded.description();
        assert!(desc.contains("text.txt"));
        assert!(desc.contains("UTF-8"));
    }
}
