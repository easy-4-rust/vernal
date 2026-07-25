//! 固定延迟周期任务测试对象。

use std::{convert::Infallible, sync::Arc, time::Duration};

use tokio_util::sync::CancellationToken;
use vernal_context::{ScheduledTask, TaskSchedule};

use super::ScheduledTaskProbe;

/// 模拟执行耗时大于调度间隔的固定延迟任务。
pub struct FixedDelayProbeTask {
    probe: Arc<ScheduledTaskProbe>,
    schedule: TaskSchedule,
    execution_time: Duration,
}

impl FixedDelayProbeTask {
    /// 创建共享探针、计划和单次执行耗时。
    pub fn new(
        probe: Arc<ScheduledTaskProbe>,
        schedule: TaskSchedule,
        execution_time: Duration,
    ) -> Self {
        Self {
            probe,
            schedule,
            execution_time,
        }
    }
}

impl ScheduledTask for FixedDelayProbeTask {
    type Error = Infallible;

    fn schedule(&self) -> TaskSchedule {
        self.schedule
    }

    async fn run(&self, cancellation: CancellationToken) -> Result<(), Self::Error> {
        self.probe.begin();
        tokio::select! {
            biased;
            () = cancellation.cancelled() => {}
            () = tokio::time::sleep(self.execution_time) => {}
        }
        self.probe.end();
        Ok(())
    }
}
