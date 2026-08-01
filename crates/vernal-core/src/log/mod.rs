//! 日志工具包。
//!
//! 对标 Spring `org.springframework.core.log` 包：`LogMessage`、`LogFormatUtils`、
//! `LogAccessor` 与 `CompositeLog`（日志门面抽象见 `crate::logging`）。

mod composite_log;
mod log_accessor;
mod log_format_utils;
mod log_message;

pub use composite_log::CompositeLog;
pub use log_accessor::LogAccessor;
pub use log_format_utils::LogFormatUtils;
pub use log_message::LogMessage;
