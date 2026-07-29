//! 解码器 trait。
//!
//! 对标 Spring `org.springframework.core.codec.Decoder`。

/// 解码器 trait。
///
/// 对应 Java: org.springframework.core.codec.Decoder
pub trait Decoder: Send + Sync {
    /// 解码器名称。
    fn name(&self) -> &str;

    /// 支持的 MIME 类型列表。
    fn supported_mime_types(&self) -> &[&str];

    /// 是否支持给定的 MIME 类型。
    fn can_decode(&self, mime_type: &str) -> bool {
        self.supported_mime_types().contains(&mime_type)
    }
}
