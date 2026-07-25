//! 周期任务不可变调度计划对象。

use std::time::Duration;

use crate::{TaskScheduleError, TaskScheduleMode};

/// 描述一个周期任务的初始延迟、执行间隔与时间基准。
///
/// 本对象不包含 Cron、时区或持久化作业语义。Vernal 只负责进程内 Tokio 周期
/// 调度；分布式选主、补偿执行、作业数据库和业务重试属于消费方或专用任务框架。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskSchedule {
    mode: TaskScheduleMode,
    initial_delay: Duration,
    interval: Duration,
}

impl TaskSchedule {
    /// 创建首次立即执行、随后按完成时间等待的固定延迟计划。
    ///
    /// # Errors
    ///
    /// `interval` 为零时返回 [`TaskScheduleError::ZeroInterval`]。
    pub fn fixed_delay(interval: Duration) -> Result<Self, TaskScheduleError> {
        Self::fixed_delay_after(Duration::ZERO, interval)
    }

    /// 创建带初始延迟的固定延迟计划。
    ///
    /// # Errors
    ///
    /// `interval` 为零时返回 [`TaskScheduleError::ZeroInterval`]。
    pub fn fixed_delay_after(
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<Self, TaskScheduleError> {
        Self::new(TaskScheduleMode::FixedDelay, initial_delay, interval)
    }

    /// 创建首次立即执行、随后沿固定时间轴触发的固定频率计划。
    ///
    /// # Errors
    ///
    /// `interval` 为零时返回 [`TaskScheduleError::ZeroInterval`]。
    pub fn fixed_rate(interval: Duration) -> Result<Self, TaskScheduleError> {
        Self::fixed_rate_after(Duration::ZERO, interval)
    }

    /// 创建带初始延迟的固定频率计划。
    ///
    /// # Errors
    ///
    /// `interval` 为零时返回 [`TaskScheduleError::ZeroInterval`]。
    pub fn fixed_rate_after(
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<Self, TaskScheduleError> {
        Self::new(TaskScheduleMode::FixedRate, initial_delay, interval)
    }

    /// 校验并创建不可变计划。
    fn new(
        mode: TaskScheduleMode,
        initial_delay: Duration,
        interval: Duration,
    ) -> Result<Self, TaskScheduleError> {
        if interval.is_zero() {
            return Err(TaskScheduleError::ZeroInterval);
        }
        Ok(Self {
            mode,
            initial_delay,
            interval,
        })
    }

    /// 返回调度时间基准。
    #[must_use]
    pub const fn mode(self) -> TaskScheduleMode {
        self.mode
    }

    /// 返回 Context 启动任务后、首次执行前的等待时间。
    #[must_use]
    pub const fn initial_delay(self) -> Duration {
        self.initial_delay
    }

    /// 返回两次计划触发之间的非零间隔。
    #[must_use]
    pub const fn interval(self) -> Duration {
        self.interval
    }
}
