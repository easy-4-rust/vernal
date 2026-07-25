//! 管理任务的附加配置选项。
//!
//! 对标 tx_di 的任务配置模式：单个任务超时、失败重试、优先级。
//! 通过 [`ManagedTaskSupervisor::spawn_with_options`] 使用。

use std::time::Duration;

/// 管理任务的附加配置选项。
///
/// 用于 [`super::ManagedTaskSupervisor::spawn_with_options`]，为单个任务
/// 提供超出默认策略的细粒度控制。
///
/// # 默认值
///
/// - `timeout`：None（跟随全局 TaskShutdownPolicy）
/// - `max_retries`：0（不重试）
/// - `priority`：TaskPriority::Normal
#[derive(Debug, Clone)]
pub struct TaskOptions {
    /// 单个任务超时。None 表示跟随全局策略。
    pub timeout: Option<Duration>,
    /// 失败重试次数。0 表示不重试。
    pub max_retries: u32,
    /// 任务优先级（影响关闭顺序：高优先级最后关闭）。
    pub priority: TaskPriority,
}

impl TaskOptions {
    /// 创建默认选项。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置单个任务超时。
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置最大重试次数。
    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置任务优先级。
    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
}

impl Default for TaskOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            max_retries: 0,
            priority: TaskPriority::Normal,
        }
    }
}

/// 任务优先级，影响关闭顺序。
///
/// 高优先级的任务在关闭时最后停止（确保关键服务最后退出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
    /// 低优先级：关闭时最先停止
    Low,
    /// 正常优先级（默认）
    Normal,
    /// 高优先级：关闭时最后停止
    High,
    /// 关键优先级：仅在所有其他任务停止后才停止
    Critical,
}
