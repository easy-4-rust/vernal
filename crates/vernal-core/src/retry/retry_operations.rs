//! 重试操作契约。
//!
//! 对标 Spring `org.springframework.core.retry.RetryOperations`。

use super::RetryPolicy;

/// 重试操作契约。
///
/// 对应 Java: org.springframework.core.retry.RetryOperations
///
/// Spring 语义：`execute` 按策略执行可重试操作，耗尽后返回最后一次错误。
pub trait RetryOperations: Send + Sync {
    /// 执行带重试的操作。
    ///
    /// # 错误
    ///
    /// 重试耗尽后返回最后一次错误。
    fn execute<F>(&self, policy: &dyn RetryPolicy, operation: F) -> Result<(), String>
    where
        F: FnMut() -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retry::RetryTemplate;

    #[test]
    fn template_satisfies_contract() {
        // D 类（重构安全）：`RetryTemplate` 实现该契约
        fn assert_operations<T: RetryOperations>() {}
        assert_operations::<RetryTemplate>();
    }
}
