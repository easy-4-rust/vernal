//! 任务执行器 trait。
//!
//! 对标 Spring `org.springframework.core.task.TaskExecutor`。

/// 任务执行器 trait。
///
/// 对应 Java: org.springframework.core.task.TaskExecutor
pub trait TaskExecutor: Send + Sync {
    /// 执行给定的同步任务。
    ///
    /// 对应 Java: `TaskExecutor#execute(Runnable)`
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);

    /// 返回任务执行器名称（用于诊断日志）。
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// 测试双：记录执行次数与名称的同步执行器。
    struct TestExecutor {
        name: String,
        executed: Arc<AtomicBool>,
    }

    impl TaskExecutor for TestExecutor {
        fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
            task();
            self.executed.store(true, Ordering::SeqCst);
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn execute_runs_the_task() {
        // A 类（合同对齐）：对标 Spring `TaskExecutor#execute(Runnable)` 必须执行任务
        let executed = Arc::new(AtomicBool::new(false));
        let executor = TestExecutor {
            name: "sync".to_string(),
            executed: executed.clone(),
        };
        let check = executed.clone();

        executor.execute(Box::new(move || {
            check.store(true, Ordering::SeqCst);
        }));

        assert!(executed.load(Ordering::SeqCst), "任务应被执行");
    }

    #[test]
    fn name_returns_configured_name() {
        // B 类（边界行为）：执行器名称用于诊断日志
        let executor = TestExecutor {
            name: "worker-1".to_string(),
            executed: Arc::new(AtomicBool::new(false)),
        };
        assert_eq!(executor.name(), "worker-1");
    }

    #[test]
    fn task_executor_is_object_safe() {
        // 对标 Spring: TaskExecutor 可被 Box 动态分发（`Executor` 注入点）
        let executor = TestExecutor {
            name: "boxed".to_string(),
            executed: Arc::new(AtomicBool::new(false)),
        };
        let boxed: Box<dyn TaskExecutor> = Box::new(executor);
        boxed.execute(Box::new(|| {}));
        assert_eq!(boxed.name(), "boxed");
    }

    #[test]
    fn multiple_executions_are_independent() {
        // C 类（生命周期）：同一执行器可重复提交任务，互不影响
        let executed = Arc::new(AtomicBool::new(false));
        let executor = TestExecutor {
            name: "reuse".to_string(),
            executed: executed.clone(),
        };
        for _ in 0..3 {
            executor.execute(Box::new(|| {}));
        }
        assert!(executed.load(Ordering::SeqCst));
    }
}
