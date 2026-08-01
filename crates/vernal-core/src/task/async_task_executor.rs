//! 异步任务执行器 trait。
//!
//! 对标 Spring `org.springframework.core.task.AsyncTaskExecutor`。

use super::task_error::TaskError;
use super::task_executor::TaskExecutor;

/// 异步任务执行器 trait。
///
/// 对应 Java: org.springframework.core.task.AsyncTaskExecutor
pub trait AsyncTaskExecutor: TaskExecutor {
    /// 异步执行给定的 Future，返回 `JoinHandle`。
    ///
    /// 对应 Java: `AsyncTaskExecutor#submit(Callable)`
    fn execute_async(
        &self,
        task: impl FnOnce() + Send + 'static,
    ) -> impl std::future::Future<Output = Result<(), TaskError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll};

    /// 测试双：立即完成的异步执行器。
    struct ImmediateExecutor {
        name: String,
        executed: Arc<AtomicBool>,
    }

    impl TaskExecutor for ImmediateExecutor {
        fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
            task();
            self.executed.store(true, Ordering::SeqCst);
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    impl AsyncTaskExecutor for ImmediateExecutor {
        fn execute_async(
            &self,
            task: impl FnOnce() + Send + 'static,
        ) -> impl Future<Output = Result<(), TaskError>> + Send {
            let executed = self.executed.clone();
            async move {
                task();
                executed.store(true, Ordering::SeqCst);
                Ok(())
            }
        }
    }

    /// 测试双：总是拒绝任务的异步执行器（对标 `TaskRejectedException`）。
    struct RejectingExecutor {
        name: String,
    }

    impl TaskExecutor for RejectingExecutor {
        fn execute(&self, _task: Box<dyn FnOnce() + Send + 'static>) {
            // 拒绝执行（同步路径不执行任务）
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    impl AsyncTaskExecutor for RejectingExecutor {
        async fn execute_async(
            &self,
            _task: impl FnOnce() + Send + 'static,
        ) -> Result<(), TaskError> {
            Err(TaskError::Rejected("queue full".to_string()))
        }
    }

    /// 零 unsafe 的最小阻塞执行器（`Waker::noop()` 自 Rust 1.85 稳定）。
    fn block_on<F: Future>(fut: F) -> F::Output {
        let waker = std::task::Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut fut = std::pin::pin!(fut);
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn execute_async_completes_task_and_returns_ok() {
        // A 类（合同对齐）：对标 Spring `AsyncTaskExecutor#submit` 成功路径
        let executed = Arc::new(AtomicBool::new(false));
        let executor = ImmediateExecutor {
            name: "async".to_string(),
            executed: executed.clone(),
        };
        let check = executed.clone();

        let result = block_on(executor.execute_async(Box::new(move || {
            check.store(true, Ordering::SeqCst);
        })));

        assert!(result.is_ok(), "成功路径应返回 Ok(())");
        assert!(executed.load(Ordering::SeqCst), "任务应被执行");
    }

    #[test]
    fn execute_async_propagates_rejection_error() {
        // B 类（错误路径）：对标 Spring `TaskRejectedException` 语义
        let executor = RejectingExecutor { name: "reject".to_string() };
        let result = block_on(executor.execute_async(Box::new(|| {})));
        assert!(
            matches!(result, Err(TaskError::Rejected(_))),
            "拒绝的任务应返回 TaskError::Rejected"
        );
    }

    #[test]
    fn async_executor_is_send_and_async_trait_composes() {
        // C 类（生命周期）：AsyncTaskExecutor 必须同时满足 TaskExecutor 合同
        let executor = ImmediateExecutor {
            name: "compose".to_string(),
            executed: Arc::new(AtomicBool::new(false)),
        };
        executor.execute(Box::new(|| {}));
        assert_eq!(executor.name(), "compose");
    }
}
