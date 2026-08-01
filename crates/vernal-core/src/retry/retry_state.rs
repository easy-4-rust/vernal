//! 重试状态契约。
//!
//! 对标 Spring `org.springframework.core.retry.RetryState`。

/// 重试状态契约。
///
/// 对应 Java: org.springframework.core.retry.RetryState
///
/// Spring 语义：有状态重试的键与结果判定（对标 `key` / `isExhausted`）。
pub trait RetryState: Send + Sync {
    /// 返回重试键。
    fn key(&self) -> &str;

    /// 是否已耗尽（不可继续重试）。
    fn is_exhausted(&self) -> bool;

    /// 标记耗尽。
    fn mark_exhausted(&mut self);
}

/// 简单键控重试状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRetryState {
    key: String,
    exhausted: bool,
}

impl SimpleRetryState {
    /// 创建状态。
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            exhausted: false,
        }
    }
}

impl RetryState for SimpleRetryState {
    fn key(&self) -> &str {
        &self.key
    }

    fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    fn mark_exhausted(&mut self) {
        self.exhausted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_key_and_exhaustion() {
        // A 类（合同对齐）：对标 Spring 状态语义
        let mut state = SimpleRetryState::new("order-42");
        assert_eq!(state.key(), "order-42");
        assert!(!state.is_exhausted());
        state.mark_exhausted();
        assert!(state.is_exhausted());
    }
}
