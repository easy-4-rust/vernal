//! 时间工具模块。
//!
//! 对标 Spring `org.springframework.util` 中与时间相关的工具:
//!
//! - [`StopWatch`]: 多任务分段计时器,对标 `StopWatch.java`
//! - [`StopWatchUnit`]: 时间单位枚举,对标 `java.util.concurrent.TimeUnit` 的子集
//!
//! # 设计原则
//!
//! 仅包含框架必须的计时工具。
//! 不包含 `DateUtil`(通用日期工具,由 hutool-rust 提供)。

mod stop_watch;
mod stop_watch_unit;

pub use stop_watch::{StopWatch, StopWatchError, TaskInfo};
pub use stop_watch_unit::StopWatchUnit;
