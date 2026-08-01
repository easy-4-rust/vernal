//! 任务回调契约。
//!
//! 对标 Spring `org.springframework.core.task.TaskCallback`。

/// 任务回调契约。
///
/// 对应 Java: org.springframework.core.task.TaskCallback
///
/// Spring 语义：异步任务完成/失败的回调（对标 Spring 6.1 的
/// `TaskCallback#onSuccess / onError`）。
pub trait TaskCallback<T>: Send + Sync {
    /// 任务成功完成回调。
    fn on_success(&self, result: &T);

    /// 任务失败回调。
    fn on_error(&self, error: &dyn std::error::Error);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct RecordingCallback {
        success: Arc<AtomicBool>,
    }

    impl TaskCallback<String> for RecordingCallback {
        fn on_success(&self, result: &String) {
            assert_eq!(result, "ok");
            self.success.store(true, Ordering::SeqCst);
        }

        fn on_error(&self, _error: &dyn std::error::Error) {}
    }

    #[test]
    fn success_callback_receives_result() {
        // A 类（合同对齐）：对标 Spring onSuccess
        let success = Arc::new(AtomicBool::new(false));
        let callback = RecordingCallback { success: success.clone() };
        callback.on_success(&"ok".to_string());
        assert!(success.load(Ordering::SeqCst));
    }

    #[test]
    fn error_callback_receives_error() {
        // C 类（错误路径）：对标 Spring onError
        struct ErrorCallback;
        impl TaskCallback<()> for ErrorCallback {
            fn on_success(&self, _result: &()) {}
            fn on_error(&self, error: &dyn std::error::Error) {
                assert_eq!(error.to_string(), "boom");
            }
        }
        let error = std::io::Error::other("boom");
        ErrorCallback.on_error(&error);
    }
}
