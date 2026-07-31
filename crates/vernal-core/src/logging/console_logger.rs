//! 控制台日志记录器。
//!
//! 对标 Spring `JavaLoggingLog`（commons-logging 适配层）。
//! 默认实现：使用 `std::fmt` 输出到 stdout/stderr。

use super::log_level::LogLevel;
use super::logger::Logger;

/// 控制台日志记录器。
///
/// 对应 Java: org.springframework.util.logging.JavaLoggingLog（简化版）
#[derive(Debug, Clone)]
pub struct ConsoleLogger {
    name: String,
    level: LogLevel,
}

impl ConsoleLogger {
    /// 创建新的控制台日志记录器。
    ///
    /// 对应 Java: `LogFactory#getLog(String)`
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: LogLevel::Info,
        }
    }

    /// 设置最低日志级别。
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// 获取当前日志级别。
    #[must_use]
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// 获取日志记录器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        if level < self.level {
            return;
        }
        println!("[{}] [{}] {}", level.as_str(), self.name, message);
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_logger_creation() {
        let logger = ConsoleLogger::new("test");
        assert_eq!(logger.name(), "test");
        assert_eq!(logger.level(), LogLevel::Info);
    }

    #[test]
    fn console_logger_set_level() {
        let mut logger = ConsoleLogger::new("test");
        logger.set_level(LogLevel::Warn);
        assert_eq!(logger.level(), LogLevel::Warn);
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Error));
    }

    /// 对标 Spring `Log.isTraceEnabled()`: 比当前级别低的不启用
    #[test]
    fn is_enabled_respects_minimum_level() {
        let mut logger = ConsoleLogger::new("threshold-test");
        logger.set_level(LogLevel::Warn);
        assert!(!logger.is_enabled(LogLevel::Trace));
        assert!(!logger.is_enabled(LogLevel::Debug));
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
    }

    /// 对标 Spring `Log.log(level, msg)`: 低于阈值的消息被丢弃
    #[test]
    fn log_below_threshold_does_not_panic() {
        // 通过不 panic 来验证: 低级别的 log 静默返回
        let mut logger = ConsoleLogger::new("silent");
        logger.set_level(LogLevel::Error);
        logger.log(LogLevel::Debug, "should be dropped");
        logger.log(LogLevel::Trace, "should be dropped");
        logger.log(LogLevel::Error, "should pass through");
    }

    /// 等级阈值: Debug 级别启用 Trace, 但禁用 Warn/Error? 不, Debug 启用 Debug 及以上
    /// 对标 Spring `Log.isDebugEnabled()`
    #[test]
    fn debug_level_enables_debug_and_above() {
        let mut logger = ConsoleLogger::new("debug-test");
        logger.set_level(LogLevel::Debug);
        assert!(!logger.is_enabled(LogLevel::Trace));
        assert!(logger.is_enabled(LogLevel::Debug));
        assert!(logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
    }
}
