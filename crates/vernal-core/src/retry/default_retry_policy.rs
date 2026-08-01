//! 默认重试策略。
//!
//! 对标 Spring `org.springframework.core.retry.DefaultRetryPolicy`。

use std::time::Duration;

use super::RetryPolicy;

/// 默认重试策略。
///
/// 对应 Java: org.springframework.core.retry.DefaultRetryPolicy
///
/// Spring 语义：允许重试（最多 3 次尝试），无退避间隔。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultRetryPolicy {
    max_attempts: u32,
}

impl DefaultRetryPolicy {
    /// 创建默认策略（最多 3 次尝试）。
    #[must_use]
    pub fn new() -> Self {
        Self { max_attempts: 3 }
    }

    /// 创建指定最大尝试次数的策略。
    #[must_use]
    pub fn with_max_attempts(max_attempts: u32) -> Self {
        Self { max_attempts }
    }
}

impl Default for DefaultRetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryPolicy for DefaultRetryPolicy {
    fn can_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    fn backoff(&self, _attempt: u32) -> Duration {
        Duration::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_allows_three_attempts() {
        // A 类（合同对齐）：对标 Spring 默认最大尝试
        let policy = DefaultRetryPolicy::new();
        assert_eq!(policy.max_attempts(), 3);
        assert!(policy.can_retry(0));
        assert!(policy.can_retry(2));
        assert!(!policy.can_retry(3));
    }

    #[test]
    fn custom_max_attempts() {
        // B 类（边界行为）
        let policy = DefaultRetryPolicy::with_max_attempts(5);
        assert!(policy.can_retry(4));
        assert!(!policy.can_retry(5));
    }
}
