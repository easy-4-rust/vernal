//! 资源解码器。
//!
//! 对标 Spring `org.springframework.core.codec.ResourceDecoder`。

use std::io;

use crate::io::{ByteArrayResource, Resource};

use super::{AbstractDecoder, Decoder};

/// 资源解码器。
///
/// 对应 Java: org.springframework.core.codec.ResourceDecoder
///
/// Spring 语义：把数据缓冲解码为 `Resource` 的抽象解码器——
/// 在 vernal 中解码结果承载于内存 [`ByteArrayResource`]。
pub struct ResourceDecoder {
    /// 支持的 MIME 类型。
    mime_types: Vec<&'static str>,
}

impl ResourceDecoder {
    /// 创建接受任意二进制内容的解码器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            mime_types: vec!["application/octet-stream", "image/png", "image/jpeg"],
        }
    }
}

impl Default for ResourceDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// 继承链标记：对标 Spring `ResourceDecoder extends AbstractDataBufferDecoder`。
impl AbstractDecoder for ResourceDecoder {}

impl Decoder for ResourceDecoder {
    fn name(&self) -> &'static str {
        "resourceDecoder"
    }

    fn supported_mime_types(&self) -> &[&str] {
        &self.mime_types
    }

    fn can_decode(&self, mime_type: &str) -> bool {
        self.mime_types.contains(&mime_type)
    }
}

impl ResourceDecoder {
    /// 把字节解码为内存资源。
    ///
    /// # 错误
    ///
    /// 解码失败时返回 [`std::io::Error`]。
    pub fn decode_resource(&self, bytes: &[u8]) -> io::Result<Box<dyn Resource>> {
        Ok(Box::new(ByteArrayResource::new(bytes.to_vec())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_bytes_into_resource() {
        // A 类（合同对齐）：对标 Spring 资源解码
        let decoder = ResourceDecoder::new();
        let resource = decoder.decode_resource(b"payload").unwrap();
        assert_eq!(resource.read_bytes().unwrap(), b"payload");
        assert!(resource.exists());
        assert_eq!(decoder.name(), "resourceDecoder");
    }

    #[test]
    fn binary_mime_types_accepted() {
        // B 类（边界行为）：二进制类型放行、文本类型拒绝
        let decoder = ResourceDecoder::new();
        assert!(decoder.can_decode("application/octet-stream"));
        assert!(!decoder.can_decode("text/plain"));
    }

    #[test]
    fn decodes_empty_payload() {
        // D 类（生命周期/边界）：空负载也产生可读资源
        let decoder = ResourceDecoder::new();
        let resource = decoder.decode_resource(b"").unwrap();
        assert!(resource.read_bytes().unwrap().is_empty());
    }
}
