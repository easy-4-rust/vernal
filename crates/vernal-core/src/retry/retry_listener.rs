//! 重试监听器契约。
//!
//! 对标 Spring `org.springframework.core.retry.RetryListener`。

/// 重试监听器契约。
///
/// 对应 Java: org.springframework.core.retry.RetryListener
///
/// Spring 语义：重试生命周期回调（开始/成功/耗尽）。
pub trait RetryListener: Send + Sync {
    /// 重试开始回调。
    fn on_start(&self, attempt: u32);

    /// 单次尝试成功回调。
    fn on_success(&self, attempt: u32);

    /// 单次尝试失败回调。
    fn on_error(&self, attempt: u32, error: &str);

    /// 重试耗尽回调。
    fn on_exhausted(&self, attempts: u32, last_error: &str);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    struct RecordingListener {
        starts: Arc<AtomicU32>,
        errors: Arc<AtomicU32>,
    }

    impl RetryListener for RecordingListener {
        fn on_start(&self, _attempt: u32) {
            self.starts.fetch_add(1, Ordering::SeqCst);
        }
        fn on_success(&self, _attempt: u32) {}
        fn on_error(&self, _attempt: u32, _error: &str) {
            self.errors.fetch_add(1, Ordering::SeqCst);
        }
        fn on_exhausted(&self, _attempts: u32, _last_error: &str) {}
    }

    #[test]
    fn receives_lifecycle_callbacks() {
        // A 类（合同对齐）：对标 Spring 回调
        let starts = Arc::new(AtomicU32::new(0));
        let errors = Arc::new(AtomicU32::new(0));
        let listener = RecordingListener {
            starts: starts.clone(),
            errors: errors.clone(),
        };
        listener.on_start(0);
        listener.on_error(1, "boom");
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        assert_eq!(errors.load(Ordering::SeqCst), 1);
    }
}
