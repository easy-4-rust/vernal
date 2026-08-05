//! `tracing` 后端日志记录器。
//!
//! 对标 Spring `commons-logging` 集成 `log4j` / `slf4j`。
//! 使用 `tracing` crate 作为后端，提供结构化日志记录。

use super::log_level::LogLevel;
use super::logger::Logger;

/// Tracing 日志记录器。
///
/// 对应 Java: org.apache.commons.logging.impl.Slf4jLogFactory
#[derive(Debug, Clone)]
pub struct TracingLogger {
    name: String,
}

impl TracingLogger {
    /// 创建新的 Tracing 日志记录器。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// 获取日志记录器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Logger for TracingLogger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Trace => tracing::trace!(logger = %self.name, "{message}"),
            LogLevel::Debug => tracing::debug!(logger = %self.name, "{message}"),
            LogLevel::Info => tracing::info!(logger = %self.name, "{message}"),
            LogLevel::Warn => tracing::warn!(logger = %self.name, "{message}"),
            LogLevel::Error => tracing::error!(logger = %self.name, "{message}"),
        }
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        match level {
            LogLevel::Trace => {
                tracing::event_enabled!(tracing::Level::TRACE, "logger" = %self.name)
            }
            LogLevel::Debug => {
                tracing::event_enabled!(tracing::Level::DEBUG, "logger" = %self.name)
            }
            LogLevel::Info => tracing::event_enabled!(tracing::Level::INFO, "logger" = %self.name),
            LogLevel::Warn => tracing::event_enabled!(tracing::Level::WARN, "logger" = %self.name),
            LogLevel::Error => {
                tracing::event_enabled!(tracing::Level::ERROR, "logger" = %self.name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_logger_creation() {
        let logger = TracingLogger::new("test");
        assert_eq!(logger.name(), "test");
    }

    #[test]
    fn tracing_logger_emits() {
        let logger = TracingLogger::new("test");
        logger.log(LogLevel::Info, "hello");
        logger.log(LogLevel::Error, "oops");
    }
}
