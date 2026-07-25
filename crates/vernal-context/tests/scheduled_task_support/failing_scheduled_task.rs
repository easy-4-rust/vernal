//! 返回业务错误的周期任务测试对象。

use std::{io, time::Duration};

use tokio_util::sync::CancellationToken;
use vernal_context::{ScheduledTask, TaskSchedule};

/// 首次执行立即返回包含敏感模拟正文的错误。
pub struct FailingScheduledTask;

impl ScheduledTask for FailingScheduledTask {
    type Error = io::Error;

    fn schedule(&self) -> TaskSchedule {
        TaskSchedule::fixed_delay(Duration::from_secs(1)).expect("valid test schedule")
    }

    async fn run(&self, _cancellation: CancellationToken) -> Result<(), Self::Error> {
        Err(io::Error::other(
            "secret-scheduled-task-downstream-response",
        ))
    }
}
