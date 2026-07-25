//! 周期任务合同测试支持对象集合。

mod failing_scheduled_task;
mod fixed_delay_probe_task;
mod fixed_rate_probe_task;
mod panicking_scheduled_task;
mod scheduled_task_application_module;
mod scheduled_task_probe;

pub use failing_scheduled_task::FailingScheduledTask;
pub use fixed_delay_probe_task::FixedDelayProbeTask;
pub use fixed_rate_probe_task::FixedRateProbeTask;
pub use panicking_scheduled_task::PanickingScheduledTask;
pub use scheduled_task_application_module::ScheduledTaskApplicationModule;
pub use scheduled_task_probe::ScheduledTaskProbe;
