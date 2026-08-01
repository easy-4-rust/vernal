//! 重试策略契约。
//!
//! 对标 Spring `org.springframework.core.retry.RetryPolicy`。

/// 重试策略契约。
///
/// 对应 Java: org.springframework.core.retry.RetryPolicy
///
/// Spring 语义：决定是否允许重试、最大尝试次数与重试间隔。
pub trait RetryPolicy: Send + Sync {
    /// 是否允许继续重试。
    ///
    /// 对应 Java: `RetryPolicy#canRetry(RetryContext)`
    fn can_retry(&self, attempt: u32) -> bool;

    /// 返回最大尝试次数。
    fn max_attempts(&self) -> u32;

    /// 返回第 `attempt` 次尝试前的等待时长（第 1 次尝试不等待）。
    fn backoff(&self, attempt: u32) -> std::time::Duration;
}

/// 固定间隔回退策略（对标 Spring `FixedBackOffPolicy` 语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedBackOff {
    interval: std::time::Duration,
}

impl FixedBackOff {
    /// 创建固定间隔回退。
    #[must_use]
    pub fn new(interval: std::time::Duration) -> Self {
        Self { interval }
    }
}

impl RetryPolicy for FixedBackOff {
    fn can_retry(&self, _attempt: u32) -> bool {
        true
    }

    fn max_attempts(&self) -> u32 {
        u32::MAX
    }

    fn backoff(&self, attempt: u32) -> std::time::Duration {
        if attempt <= 1 {
            std::time::Duration::ZERO
        } else {
            self.interval
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_backoff_first_attempt_immediate() {
        // A 类（合同对齐）：对标 Spring 首次尝试不等待
        let policy = FixedBackOff::new(std::time::Duration::from_millis(100));
        assert_eq!(policy.backoff(1), std::time::Duration::ZERO);
        assert_eq!(policy.backoff(2), std::time::Duration::from_millis(100));
        assert!(policy.can_retry(5));
    }
}
