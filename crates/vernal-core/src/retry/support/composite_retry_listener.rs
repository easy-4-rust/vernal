//! 组合重试监听器。
//!
//! 对标 Spring `org.springframework.core.retry.support.CompositeRetryListener`。

use crate::retry::RetryListener;

/// 组合重试监听器。
///
/// 对应 Java: org.springframework.core.retry.support.CompositeRetryListener
///
/// Spring 语义：把回调广播给多个监听器。
pub struct CompositeRetryListener {
    listeners: Vec<Box<dyn RetryListener>>,
}

impl CompositeRetryListener {
    /// 创建空组合监听器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// 追加监听器。
    pub fn add_listener(&mut self, listener: Box<dyn RetryListener>) {
        self.listeners.push(listener);
    }
}

impl Default for CompositeRetryListener {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryListener for CompositeRetryListener {
    fn on_start(&self, attempt: u32) {
        for listener in &self.listeners {
            listener.on_start(attempt);
        }
    }

    fn on_success(&self, attempt: u32) {
        for listener in &self.listeners {
            listener.on_success(attempt);
        }
    }

    fn on_error(&self, attempt: u32, error: &str) {
        for listener in &self.listeners {
            listener.on_error(attempt, error);
        }
    }

    fn on_exhausted(&self, attempts: u32, last_error: &str) {
        for listener in &self.listeners {
            listener.on_exhausted(attempts, last_error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    struct Counter {
        count: Arc<AtomicU32>,
    }

    impl RetryListener for Counter {
        fn on_start(&self, _attempt: u32) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_success(&self, _attempt: u32) {}
        fn on_error(&self, _attempt: u32, _error: &str) {}
        fn on_exhausted(&self, _attempts: u32, _last_error: &str) {}
    }

    #[test]
    fn broadcasts_to_all_listeners() {
        // A 类（合同对齐）：对标 Spring 广播语义
        let first = Arc::new(AtomicU32::new(0));
        let second = Arc::new(AtomicU32::new(0));
        let mut composite = CompositeRetryListener::new();
        composite.add_listener(Box::new(Counter { count: first.clone() }));
        composite.add_listener(Box::new(Counter { count: second.clone() }));
        composite.on_start(0);
        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
    }
}
