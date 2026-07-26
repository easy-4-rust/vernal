//! 简化版 StopWatch。
//!
//! 对标 Spring 的 `org.springframework.util.StopWatch`。
//! 仅用于 AOP 计时和性能诊断，不提供完整的日期工具。

use std::time::{Duration, Instant};

/// 简化版 StopWatch。
///
/// 用于测量代码块的执行时间，支持多个任务的分段计时。
/// 对标 Spring 的 `StopWatch`。
///
/// # 示例
///
/// ```rust
/// use vernal_core::time::StopWatch;
///
/// let mut sw = StopWatch::new("my-operation");
/// sw.start("task-1");
/// // ... 执行任务 1 ...
/// sw.stop();
/// sw.start("task-2");
/// // ... 执行任务 2 ...
/// sw.stop();
/// println!("{}", sw.pretty_print());
/// ```
pub struct StopWatch {
    /// 操作名称
    name: String,
    /// 总开始时间
    total_start: Instant,
    /// 任务列表
    tasks: Vec<TaskInfo>,
    /// 当前任务开始时间（如果有）
    current_start: Option<Instant>,
    /// 当前任务名称
    current_name: Option<String>,
}

/// 任务信息。
struct TaskInfo {
    /// 任务名称
    name: String,
    /// 任务耗时
    duration: Duration,
}

impl StopWatch {
    /// 创建新的 StopWatch。
    ///
    /// # 参数
    /// - `name`：操作名称（用于日志输出）
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            total_start: Instant::now(),
            tasks: Vec::new(),
            current_start: None,
            current_name: None,
        }
    }

    /// 开始一个新任务。
    ///
    /// # 参数
    /// - `name`：任务名称
    pub fn start(&mut self, name: impl Into<String>) {
        if let Some(start) = self.current_start {
            // 自动停止上一个任务
            let duration = start.elapsed();
            self.tasks.push(TaskInfo {
                name: self.current_name.take().unwrap_or_default(),
                duration,
            });
        }
        self.current_name = Some(name.into());
        self.current_start = Some(Instant::now());
    }

    /// 停止当前任务。
    pub fn stop(&mut self) {
        if let Some(start) = self.current_start.take() {
            let duration = start.elapsed();
            self.tasks.push(TaskInfo {
                name: self.current_name.take().unwrap_or_default(),
                duration,
            });
        }
    }

    /// 获取总耗时。
    #[must_use]
    pub fn total_elapsed(&self) -> Duration {
        self.total_start.elapsed()
    }

    /// 获取任务数量。
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len() + if self.current_start.is_some() { 1 } else { 0 }
    }

    /// 获取所有任务的耗时列表。
    #[must_use]
    pub fn task_infos(&self) -> Vec<(&str, Duration)> {
        self.tasks
            .iter()
            .map(|t| (t.name.as_str(), t.duration))
            .collect()
    }

    /// 格式化输出（类似 Spring 的 `prettyPrint()`）。
    #[must_use]
    pub fn pretty_print(&self) -> String {
        let mut result = format!("StopWatch '{}': running\n", self.name);
        for task in &self.tasks {
            result.push_str(&format!(
                "  {} took {:.3}ms\n",
                task.name,
                task.duration.as_secs_f64() * 1000.0
            ));
        }
        result.push_str(&format!(
            "Total: {:.3}ms",
            self.total_elapsed().as_secs_f64() * 1000.0
        ));
        result
    }
}

impl std::fmt::Display for StopWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}
