//! 发生 panic 的周期任务测试对象。

use std::{convert::Infallible, time::Duration};

use tokio_util::sync::CancellationToken;
use vernal_context::{ScheduledTask, TaskSchedule};

/// 首次执行立即 panic，用于验证监督器不会让 Context 失去关闭路径。
pub struct PanickingScheduledTask;

impl ScheduledTask for PanickingScheduledTask {
    type Error = Infallible;

    fn schedule(&self) -> TaskSchedule {
        TaskSchedule::fixed_delay(Duration::from_secs(1)).expect("valid test schedule")
    }

    async fn run(&self, _cancellation: CancellationToken) -> Result<(), Self::Error> {
        panic!("scheduled task panic")
    }
}
