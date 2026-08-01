//! 抽象编码器契约。
//!
//! 对标 Spring `org.springframework.core.codec.AbstractEncoder`。

use std::io;

/// 抽象编码器契约。
///
/// 对应 Java: org.springframework.core.codec.AbstractEncoder
///
/// Spring 语义：`Encoder` 的抽象基类——提供默认编码实现与模板方法；
/// 可编码性判定（`canEncode`）在 vernal 中由 [`super::Encoder`] trait 的
/// `supported_mime_types` / `can_encode` 承担，避免方法名重复导致歧义。
pub trait AbstractEncoder: Send + Sync {
    /// 把值编码为字节。
    ///
    /// # 错误
    ///
    /// 编码失败时返回 [`std::io::Error`]。
    fn encode<T: std::fmt::Display>(&self, value: &T) -> io::Result<Vec<u8>> {
        Ok(value.to_string().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TextEncoder;

    impl AbstractEncoder for TextEncoder {}

    #[test]
    fn default_encode_uses_display() {
        // A 类（合同对齐）：对标 Spring 默认编码路径
        let encoder = TextEncoder;
        assert_eq!(encoder.encode(&42_i32).unwrap(), b"42");
    }
}
