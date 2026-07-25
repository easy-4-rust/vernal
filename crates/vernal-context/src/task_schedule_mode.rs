//! 周期任务调度模式对象。

/// 定义周期任务两次执行之间的时间基准。
///
/// 两种模式都保证同一个 [`crate::ScheduledTask`] 不会并发重入。不同任务仍由
/// Tokio 独立调度，可以并发执行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskScheduleMode {
    /// 上一次执行完成后，再等待固定间隔。
    FixedDelay,
    /// 按固定时间轴触发；任务执行过慢时跳过错过的时刻，不突发补跑。
    FixedRate,
}
