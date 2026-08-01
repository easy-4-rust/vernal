//! 抽象单值编码器契约。
//!
//! 对标 Spring `org.springframework.core.codec.AbstractSingleValueEncoder`。

use std::io;

use super::AbstractEncoder;

/// 抽象单值编码器契约。
///
/// 对应 Java: org.springframework.core.codec.AbstractSingleValueEncoder
///
/// Spring 语义：面向单值的 `AbstractEncoder` 子类——一次编码一个值。
pub trait AbstractSingleValueEncoder: AbstractEncoder {
    /// 编码单值为字节（默认实现委托 `encode`）。
    ///
    /// # 错误
    ///
    /// 编码失败时返回 [`std::io::Error`]。
    fn encode_value<T: std::fmt::Display>(&self, value: &T) -> io::Result<Vec<u8>> {
        self.encode(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SingleValueEncoder;

    impl AbstractEncoder for SingleValueEncoder {}
    impl AbstractSingleValueEncoder for SingleValueEncoder {}

    #[test]
    fn encodes_single_value() {
        // A 类（合同对齐）：对标 Spring 单值编码
        let encoder = SingleValueEncoder;
        assert_eq!(encoder.encode_value(&"x").unwrap(), b"x");
    }
}
