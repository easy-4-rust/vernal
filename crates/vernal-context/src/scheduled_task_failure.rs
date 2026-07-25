//! 周期任务脱敏失败对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

/// 保存周期任务的静态身份和原始错误链，同时约束默认日志输出。
///
/// 默认 `Display`/`Debug` 不复制业务错误正文。需要根因的调用方必须显式沿
/// [`Error::source`] 读取，避免 Token、连接信息或下游响应进入普通 Context 日志。
#[derive(Clone)]
pub struct ScheduledTaskFailure {
    task: &'static str,
    source: SharedError,
}

impl ScheduledTaskFailure {
    /// 创建保留根因但默认脱敏的周期任务失败。
    #[must_use]
    pub(crate) const fn new(task: &'static str, source: SharedError) -> Self {
        Self { task, source }
    }

    /// 返回任务提供的低基数静态名称。
    #[must_use]
    pub const fn task(&self) -> &'static str {
        self.task
    }
}

impl fmt::Display for ScheduledTaskFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "scheduled task {} failed", self.task)
    }
}

impl fmt::Debug for ScheduledTaskFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScheduledTaskFailure")
            .field("task", &self.task)
            .field("source", &"[redacted; use Error::source]")
            .finish()
    }
}

impl Error for ScheduledTaskFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
