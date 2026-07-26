//! 时间工具模块。
//!
//! 仅包含框架必须的 `StopWatch`（AOP 计时）。
//! 不包含 `DateUtil`（通用日期工具，由 hutool-rust 提供）。

mod stop_watch;

pub use stop_watch::StopWatch;
