//! 编码器 trait。
//!
//! 对标 Spring `org.springframework.core.codec.Encoder`。

/// 编码器 trait。
///
/// 对应 Java: org.springframework.core.codec.Encoder
pub trait Encoder: Send + Sync {
    /// 编码器名称。
    fn name(&self) -> &str;

    /// 支持的 MIME 类型列表。
    fn supported_mime_types(&self) -> &[&str];

    /// 是否支持给定的 MIME 类型。
    fn can_encode(&self, mime_type: &str) -> bool {
        self.supported_mime_types().contains(&mime_type)
    }
}
