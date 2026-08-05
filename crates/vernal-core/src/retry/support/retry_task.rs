//! 重试任务。
//!
//! 对标 Spring `org.springframework.core.retry.support.RetryTask`。

use crate::retry::RetryPolicy;

/// 重试任务。
///
/// 对应 Java: org.springframework.core.retry.support.RetryTask
///
/// Spring 语义：把操作与重试策略绑定为可执行任务（对标 Spring 内部把
/// `RetryCallback` 包装进 `RetryTemplate.execute` 的形态）。
pub struct RetryTask<F> {
    policy: Box<dyn RetryPolicy>,
    operation: F,
}

impl<F> RetryTask<F>
where
    F: FnMut() -> Result<(), String>,
{
    /// 创建重试任务。
    #[must_use]
    pub fn new(policy: Box<dyn RetryPolicy>, operation: F) -> Self {
        Self { policy, operation }
    }

    /// 执行任务（重试耗尽返回最后一次错误）。
    ///
    /// # 错误
    ///
    /// 重试耗尽后返回最后一次错误。
    pub fn run(&mut self) -> Result<(), String> {
        let mut attempt = 0_u32;
        loop {
            match (self.operation)() {
                Ok(()) => return Ok(()),
                Err(err) => {
                    attempt += 1;
                    if !self.policy.can_retry(attempt) {
                        return Err(err);
                    }
                    let wait = self.policy.backoff(attempt);
                    if !wait.is_zero() {
                        std::thread::sleep(wait);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retry::DefaultRetryPolicy;

    #[test]
    fn runs_until_success() {
        // A 类（合同对齐）：对标 Spring 任务执行
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        let attempts = Arc::new(AtomicU32::new(0));
        let counter = attempts.clone();
        let mut task = RetryTask::new(Box::new(DefaultRetryPolicy::new()), move || {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err("transient".to_string())
            } else {
                Ok(())
            }
        });
        assert!(task.run().is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn exhausted_returns_error() {
        // C 类（错误路径）
        let mut task = RetryTask::new(Box::new(DefaultRetryPolicy::new()), || {
            Err("permanent".to_string())
        });
        assert_eq!(task.run().unwrap_err(), "permanent");
    }
}
