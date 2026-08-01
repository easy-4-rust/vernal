//! 资源区域编码器。
//!
//! 对标 Spring `org.springframework.core.codec.ResourceRegionEncoder`。

use std::io;

use crate::io::support::ResourceRegion;

use super::{AbstractEncoder, AbstractSingleValueEncoder, Encoder};

/// 资源区域编码器。
///
/// 对应 Java: org.springframework.core.codec.ResourceRegionEncoder
///
/// Spring 语义：把 `ResourceRegion`（资源区间）编码为数据缓冲的编码器，
/// 用于 HTTP Range 等区间响应场景。
pub struct ResourceRegionEncoder {
    /// 支持的 MIME 类型。
    mime_types: Vec<&'static str>,
}

impl ResourceRegionEncoder {
    /// 创建接受任意二进制内容的编码器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            mime_types: vec!["application/octet-stream", "image/png", "image/jpeg"],
        }
    }
}

impl Default for ResourceRegionEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// 继承链标记：对标 Spring `ResourceRegionEncoder extends AbstractSingleValueEncoder`。
impl AbstractEncoder for ResourceRegionEncoder {}

impl AbstractSingleValueEncoder for ResourceRegionEncoder {}

impl Encoder for ResourceRegionEncoder {
    fn name(&self) -> &'static str {
        "resourceRegionEncoder"
    }

    fn supported_mime_types(&self) -> &[&str] {
        &self.mime_types
    }

    fn can_encode(&self, mime_type: &str) -> bool {
        self.mime_types.contains(&mime_type)
    }
}

impl ResourceRegionEncoder {
    /// 读取资源区域内容为字节。
    ///
    /// # 错误
    ///
    /// 区域读取失败时返回 [`std::io::Error`]。
    pub fn encode_region(&self, region: &ResourceRegion) -> io::Result<Vec<u8>> {
        region.read_region()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn encodes_region_content() {
        // A 类（合同对齐）：对标 Spring 区域编码（HTTP Range 语义）
        let encoder = ResourceRegionEncoder::new();
        let resource = ByteArrayResource::new(b"0123456789".to_vec());
        let region = ResourceRegion::new(Box::new(resource), 2, 4);
        assert_eq!(encoder.encode_region(&region).unwrap(), b"2345");
        assert_eq!(encoder.name(), "resourceRegionEncoder");
    }

    #[test]
    fn mime_type_gate() {
        // B 类（边界行为）：二进制类型放行
        let encoder = ResourceRegionEncoder::new();
        assert!(encoder.can_encode("application/octet-stream"));
        assert!(!encoder.can_encode("text/plain"));
    }

    #[test]
    fn full_length_region_encodes_everything() {
        // D 类（生命周期/边界）：整个资源作为一个区域
        let encoder = ResourceRegionEncoder::new();
        let resource = ByteArrayResource::new(b"0123456789".to_vec());
        let region = ResourceRegion::new(Box::new(resource), 0, 10);
        assert_eq!(encoder.encode_region(&region).unwrap(), b"0123456789");
    }
}
