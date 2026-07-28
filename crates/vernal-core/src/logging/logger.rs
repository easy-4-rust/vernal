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
