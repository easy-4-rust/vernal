//! 资源编码器。
//!
//! 对标 Spring `org.springframework.core.codec.ResourceEncoder`。

use std::io;

use crate::io::Resource;

use super::{AbstractEncoder, AbstractSingleValueEncoder, Encoder};

/// 资源编码器。
///
/// 对应 Java: org.springframework.core.codec.ResourceEncoder
///
/// Spring 语义：把 `Resource` 内容编码为数据缓冲的编码器
/// （对标 `AbstractSingleValueEncoder<Resource>`）。
pub struct ResourceEncoder {
    /// 支持的 MIME 类型。
    mime_types: Vec<&'static str>,
}

impl ResourceEncoder {
    /// 创建接受任意二进制内容的编码器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            mime_types: vec!["application/octet-stream", "image/png", "image/jpeg"],
        }
    }
}

impl Default for ResourceEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// 继承链标记：对标 Spring `ResourceEncoder extends AbstractSingleValueEncoder`。
impl AbstractEncoder for ResourceEncoder {}

impl AbstractSingleValueEncoder for ResourceEncoder {}

impl Encoder for ResourceEncoder {
    fn name(&self) -> &'static str {
        "resourceEncoder"
    }

    fn supported_mime_types(&self) -> &[&str] {
        &self.mime_types
    }

    fn can_encode(&self, mime_type: &str) -> bool {
        self.mime_types.contains(&mime_type)
    }
}

impl ResourceEncoder {
    /// 读取资源内容为字节。
    ///
    /// # 错误
    ///
    /// 资源不可读时返回 [`std::io::Error`]。
    pub fn encode_resource(&self, resource: &dyn Resource) -> io::Result<Vec<u8>> {
        if !resource.is_readable() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("资源不可读: {}", resource.description()),
            ));
        }
        resource.read_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn encodes_resource_content() {
        // A 类（合同对齐）：对标 Spring 资源编码
        let encoder = ResourceEncoder::new();
        let resource = ByteArrayResource::new(b"content".to_vec());
        assert_eq!(encoder.encode_resource(&resource).unwrap(), b"content");
        assert_eq!(encoder.name(), "resourceEncoder");
    }

    #[test]
    fn mime_type_gate() {
        // B 类（边界行为）：二进制类型放行
        let encoder = ResourceEncoder::new();
        assert!(encoder.can_encode("image/png"));
        assert!(!encoder.can_encode("text/plain"));
    }

    #[test]
    fn unreadable_resource_returns_error() {
        // C 类（错误路径）：不可读资源被拒绝
        let encoder = ResourceEncoder::new();
        let resource = UnreadableResource;
        assert!(encoder.encode_resource(&resource).is_err());
    }

    struct UnreadableResource;

    impl Resource for UnreadableResource {
        fn exists(&self) -> bool {
            true
        }

        fn is_readable(&self) -> bool {
            false
        }

        fn filename(&self) -> Option<&str> {
            None
        }

        fn description(&self) -> String {
            "不可读资源".to_string()
        }

        fn read_bytes(&self) -> io::Result<Vec<u8>> {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "拒绝读取"))
        }
    }
}
