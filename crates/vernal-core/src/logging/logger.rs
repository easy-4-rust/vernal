//! 日志记录器 trait。
//!
//! 对标 Spring `org.springframework.util.logging.Logger`（commons-logging 适配层）。
//! 提供统一的日志接口，由具体 Logger 实现决定输出方式。

/// 日志记录器 trait。
///
/// 对标 Java: org.springframework.util.logging.Logger
pub trait Logger: Send + Sync {
    /// 在指定级别记录一条消息。
    ///
    /// 对应 Java: `Logger#log`
    fn log(&self, level: super::log_level::LogLevel, message: &str);

    /// trace 级别日志（最低）。
    fn trace(&self, message: &str) {
        self.log(super::log_level::LogLevel::Trace, message);
    }

    /// debug 级别日志。
    fn debug(&self, message: &str) {
        self.log(super::log_level::LogLevel::Debug, message);
    }

    /// info 级别日志。
    fn info(&self, message: &str) {
        self.log(super::log_level::LogLevel::Info, message);
    }

    /// warn 级别日志。
    fn warn(&self, message: &str) {
        self.log(super::log_level::LogLevel::Warn, message);
    }

    /// error 级别日志（最高）。
    fn error(&self, message: &str) {
        self.log(super::log_level::LogLevel::Error, message);
    }

    /// 检查给定级别是否启用（默认全部启用）。
    fn is_enabled(&self, _level: super::log_level::LogLevel) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::LogLevel;
    use std::sync::{Arc, Mutex};

    /// 捕获所有 log 调用的测试用 Logger
    /// （对标 Spring 的 `AbstractLog` 测试场景）
    #[derive(Default)]
    struct CapturingLogger {
        captured: Arc<Mutex<Vec<(LogLevel, String)>>>,
    }

    impl CapturingLogger {
        fn new() -> Self {
            Self::default()
        }

        fn snapshot(&self) -> Vec<(LogLevel, String)> {
            self.captured.lock().unwrap().clone()
        }
    }

    impl Logger for CapturingLogger {
        fn log(&self, level: LogLevel, message: &str) {
            self.captured
                .lock()
                .unwrap()
                .push((level, message.to_string()));
        }
    }

    /// trace 级别应委托给 log(Trace, ...)
    #[test]
    fn trace_default_method_delegates_to_log() {
        let l = CapturingLogger::new();
        l.trace("t-msg");
        let cap = l.snapshot();
        assert_eq!(cap.len(), 1);
        assert_eq!(cap[0].0, LogLevel::Trace);
        assert_eq!(cap[0].1, "t-msg");
    }

    /// debug 级别应委托给 log(Debug, ...)
    #[test]
    fn debug_default_method_delegates_to_log() {
        let l = CapturingLogger::new();
        l.debug("d-msg");
        let cap = l.snapshot();
        assert_eq!(cap[0].0, LogLevel::Debug);
        assert_eq!(cap[0].1, "d-msg");
    }

    /// info 级别应委托给 log(Info, ...)
    #[test]
    fn info_default_method_delegates_to_log() {
        let l = CapturingLogger::new();
        l.info("i-msg");
        let cap = l.snapshot();
        assert_eq!(cap[0].0, LogLevel::Info);
        assert_eq!(cap[0].1, "i-msg");
    }

    /// warn 级别应委托给 log(Warn, ...)
    #[test]
    fn warn_default_method_delegates_to_log() {
        let l = CapturingLogger::new();
        l.warn("w-msg");
        let cap = l.snapshot();
        assert_eq!(cap[0].0, LogLevel::Warn);
        assert_eq!(cap[0].1, "w-msg");
    }

    /// error 级别应委托给 log(Error, ...)
    #[test]
    fn error_default_method_delegates_to_log() {
        let l = CapturingLogger::new();
        l.error("e-msg");
        let cap = l.snapshot();
        assert_eq!(cap[0].0, LogLevel::Error);
        assert_eq!(cap[0].1, "e-msg");
    }

    /// 默认 is_enabled 对所有级别都返回 true
    /// （对标 Spring `Logger.isInfoEnabled()` 等方法默认实现）
    #[test]
    fn is_enabled_default_returns_true_for_all_levels() {
        let l = CapturingLogger::new();
        assert!(l.is_enabled(LogLevel::Trace));
        assert!(l.is_enabled(LogLevel::Debug));
        assert!(l.is_enabled(LogLevel::Info));
        assert!(l.is_enabled(LogLevel::Warn));
        assert!(l.is_enabled(LogLevel::Error));
    }
}
