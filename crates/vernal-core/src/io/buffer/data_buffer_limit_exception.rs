//! 数据缓冲超限异常。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferLimitException`。

use std::fmt;

/// 数据缓冲超限异常。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferLimitException
///
/// Spring 语义：缓冲总大小超过限制时抛出的运行时异常——
/// 记录超限上限与超限描述。
#[derive(Debug)]
pub struct DataBufferLimitException {
    message: String,
    max_limit: usize,
}

impl DataBufferLimitException {
    /// 创建异常。
    #[must_use]
    pub fn new(max_limit: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            max_limit,
        }
    }

    /// 超限上限（对标 Spring `getMaxLimit`）。
    #[must_use]
    pub fn max_limit(&self) -> usize {
        self.max_limit
    }
}

impl fmt::Display for DataBufferLimitException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "超出数据缓冲上限 {}: {}", self.max_limit, self.message)
    }
}

impl std::error::Error for DataBufferLimitException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_limit_and_message() {
        // A 类（合同对齐）：对标 Spring 异常信息
        let err = DataBufferLimitException::new(256, "请求体过大");
        assert_eq!(err.max_limit(), 256);
        assert!(err.to_string().contains("256"));
        assert!(err.to_string().contains("请求体过大"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<DataBufferLimitException>();
    }
}
