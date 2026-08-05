//! 简单任务执行器（同步）。
//!
//! 对标 Spring `org.springframework.core.task.SyncTaskExecutor`。

use super::task_executor::TaskExecutor;

/// 简单同步任务执行器。
///
/// 对应 Java: org.springframework.core.task.SyncTaskExecutor
#[derive(Debug, Default)]
pub struct SimpleTaskExecutor {
    name: String,
}

impl SimpleTaskExecutor {
    /// 创建默认执行器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "simple".to_string(),
        }
    }

    /// 创建带名称的执行器。
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl TaskExecutor for SimpleTaskExecutor {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        task();
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn new_creates_executor_with_default_name() {
        // 对标 Spring `SyncTaskExecutor()` 默认构造（无名称）
        let executor = SimpleTaskExecutor::new();
        assert_eq!(executor.name(), "simple");
    }

    #[test]
    fn with_name_creates_executor_with_custom_name() {
        // 对标 Spring 通过 ThreadPoolTaskExecutor 设置 threadNamePrefix
        let executor = SimpleTaskExecutor::with_name("worker-1");
        assert_eq!(executor.name(), "worker-1");
    }

    #[test]
    fn with_name_accepts_string_and_str() {
        // 验证 `impl Into<String>` 接受 &str 和 String
        let from_str = SimpleTaskExecutor::with_name("from-str");
        let from_string = SimpleTaskExecutor::with_name(String::from("from-string"));
        assert_eq!(from_str.name(), "from-str");
        assert_eq!(from_string.name(), "from-string");
    }

    #[test]
    fn execute_runs_task_synchronously() {
        // 对标 Spring `SyncTaskExecutor.execute(Runnable)` 同步在同一线程运行
        let executor = SimpleTaskExecutor::new();
        let counter = std::sync::Arc::new(AtomicI32::new(0));
        let counter_clone = std::sync::Arc::clone(&counter);
        executor.execute(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn execute_runs_multiple_tasks_in_order() {
        let executor = SimpleTaskExecutor::new();
        let log: std::sync::Arc<std::sync::Mutex<Vec<i32>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        for i in 0..5 {
            let log_clone = std::sync::Arc::clone(&log);
            let value = i;
            executor.execute(Box::new(move || {
                log_clone.lock().unwrap().push(value);
            }));
        }
        let snapshot = log.lock().unwrap();
        assert_eq!(*snapshot, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn execute_supports_capturing_closure_state() {
        // 验证闭包可以捕获外部变量
        let executor = SimpleTaskExecutor::new();
        let result: std::sync::Arc<std::sync::Mutex<Option<i32>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let result_clone = std::sync::Arc::clone(&result);
        let multiplier = 7;
        let input = 6;
        executor.execute(Box::new(move || {
            *result_clone.lock().unwrap() = Some(input * multiplier);
        }));
        assert_eq!(*result.lock().unwrap(), Some(42));
    }

    #[test]
    fn name_method_returns_borrowed_str() {
        // 验证 name() 返回 &str 而非 String（避免分配）
        let executor = SimpleTaskExecutor::with_name("fast");
        let name: &str = executor.name();
        assert_eq!(name, "fast");
        // 多次调用都应返回同一内部字符串
        assert_eq!(executor.name(), executor.name());
    }

    #[test]
    fn executor_is_send_and_sync() {
        // 对标 Spring `TaskExecutor` 接口要求跨线程安全
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SimpleTaskExecutor>();
    }

    #[test]
    fn executor_via_trait_object_runs_task() {
        // 通过 trait object 调用, 验证 trait 抽象
        let executor: Box<dyn TaskExecutor> = Box::new(SimpleTaskExecutor::with_name("dyn"));
        assert_eq!(executor.name(), "dyn");
        let counter = std::sync::Arc::new(AtomicI32::new(10));
        let counter_for_task = std::sync::Arc::clone(&counter);
        executor.execute(Box::new(move || {
            counter_for_task.fetch_add(5, Ordering::SeqCst);
        }));
        assert_eq!(counter.load(Ordering::SeqCst), 15);
    }
}
