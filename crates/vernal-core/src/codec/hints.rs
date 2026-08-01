//! 编解码提示常量。
//!
//! 对标 Spring `org.springframework.core.codec.Hints`。

/// 编解码提示常量。
///
/// 对应 Java: org.springframework.core.codec.Hints
///
/// Spring 语义：编解码器的提示键常量集合（日志前缀、压制日志等）。
pub struct Hints;

impl Hints {
    /// 日志前缀提示键（对标 Spring `LOG_PREFIX_HINT`）。
    pub const LOG_PREFIX_HINT: &'static str = "logPrefix";

    /// 压制日志提示键（对标 Spring `SUPPRESS_LOGGING_HINT`）。
    pub const SUPPRESS_LOGGING_HINT: &'static str = "suppressLogging";

    /// 全部提示键。
    #[must_use]
    pub const fn all() -> &'static [&'static str] {
        &[Self::LOG_PREFIX_HINT, Self::SUPPRESS_LOGGING_HINT]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_hint_keys() {
        // A 类（合同对齐）：对标 Spring 提示键常量
        assert_eq!(Hints::LOG_PREFIX_HINT, "logPrefix");
        assert_eq!(Hints::SUPPRESS_LOGGING_HINT, "suppressLogging");
        assert_eq!(Hints::all().len(), 2);
    }
}
