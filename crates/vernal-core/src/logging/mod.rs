//! 日志门面模块。
//!
//! 对标 Spring `org.springframework.util.logging` 与 `commons-logging` 适配层。
//! 提供统一的 `LogLevel` 与 `Logger` trait，输出由具体 Logger 实现决定。

mod console_logger;
mod log_level;
mod logger;
mod noop_logger;

#[cfg(feature = "tracing")]
mod tracing_logger;

pub use console_logger::ConsoleLogger;
pub use log_level::LogLevel;
pub use logger::Logger;
pub use noop_logger::NoOpLogger;

#[cfg(feature = "tracing")]
pub use tracing_logger::TracingLogger;
