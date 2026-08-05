//! 重试模板。
//!
//! 对标 Spring `org.springframework.core.retry.RetryTemplate`。

use super::RetryPolicy;
use super::default_retry_policy::DefaultRetryPolicy;
use super::retry_operations::RetryOperations;

/// 重试模板。
///
/// 对应 Java: org.springframework.core.retry.RetryTemplate
///
/// Spring 语义：按 [`RetryPolicy`] 执行操作，重试耗尽后返回最后一次错误。
pub struct RetryTemplate {
    policy: Box<dyn RetryPolicy>,
}

impl RetryTemplate {
    /// 创建使用默认策略（3 次尝试）的模板。
    #[must_use]
    pub fn new() -> Self {
        Self {
            policy: Box::new(DefaultRetryPolicy::new()),
        }
    }

    /// 创建使用指定策略的模板。
    #[must_use]
    pub fn with_policy(policy: Box<dyn RetryPolicy>) -> Self {
        Self { policy }
    }
}

impl Default for RetryTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryOperations for RetryTemplate {
    fn execute<F>(&self, policy: &dyn RetryPolicy, mut operation: F) -> Result<(), String>
    where
        F: FnMut() -> Result<(), String>,
    {
        let mut attempt = 0_u32;
        loop {
            match operation() {
                Ok(()) => return Ok(()),
                Err(err) => {
                    attempt += 1;
                    if policy.can_retry(attempt) {
                        let wait = policy.backoff(attempt);
                        if !wait.is_zero() {
                            std::thread::sleep(wait);
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}

impl RetryTemplate {
    /// 便捷：使用内置策略直接执行。
    ///
    /// # 错误
    ///
    /// 重试耗尽后返回最后一次错误。
    pub fn execute_with_default<F>(&self, operation: F) -> Result<(), String>
    where
        F: FnMut() -> Result<(), String>,
    {
        self.execute(self.policy.as_ref(), operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn succeeds_on_first_attempt() {
        // A 类（合同对齐）：对标 Spring 首次成功
        let template = RetryTemplate::new();
        assert!(template.execute_with_default(|| Ok(())).is_ok());
    }

    #[test]
    fn retries_until_success() {
        // A 类（合同对齐）：对标 Spring 重试直到成功
        let attempts = Arc::new(AtomicU32::new(0));
        let template = RetryTemplate::new();
        let counter = attempts.clone();
        let result = template.execute_with_default(move || {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err("transient".to_string())
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn exhausted_returns_last_error() {
        // C 类（错误路径）：对标 Spring 重试耗尽
        let template = RetryTemplate::new();
        let result = template.execute_with_default(|| Err("permanent".to_string()));
        assert_eq!(result.unwrap_err(), "permanent");
    }
}
