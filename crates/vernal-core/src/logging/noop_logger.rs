//! 空操作日志记录器。
//!
//! 对标 Spring `NoOpLog`（commons-logging `Log` 适配层）。
//! 所有日志调用直接丢弃，用于测试和静默场景。

use super::logger::Logger;
use super::log_level::LogLevel;

/// 空操作日志记录器。
///
/// 对应 Java: org.springframework.util.logging.NoOpLog
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn log(&self, _level: LogLevel, _message: &str) {
        // 故意为空
    }

    fn is_enabled(&self, _level: LogLevel) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_does_nothing() {
        let logger = NoOpLogger;
        logger.log(LogLevel::Error, "should be discarded");
        assert!(!logger.is_enabled(LogLevel::Error));
    }

    #[test]
    fn noop_default() {
        let logger = NoOpLogger;
        logger.info("test");
    }
}
