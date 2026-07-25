//! 周期任务调度声明错误对象。

use std::{error::Error, fmt};

/// 创建无法安全驱动的周期任务计划时返回的结构化错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskScheduleError {
    /// 周期间隔为零，会造成无界忙循环。
    ZeroInterval,
}

impl fmt::Display for TaskScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroInterval => formatter.write_str("scheduled task interval must be non-zero"),
        }
    }
}

impl Error for TaskScheduleError {}
