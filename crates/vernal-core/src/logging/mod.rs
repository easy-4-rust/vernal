//! 日志抽象模块（feature = "tracing"）。
//!
//! 对标 Spring `commons-logging`（Apache Commons Logging）。
//!
//! Spring 使用 `commons-logging` 作为日志门面，vernal-core 使用 `tracing` 作为日志门面。
//! `tracing` 是 Rust 生态的事实标准日志框架，由 Tokio 团队维护。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `org.apache.commons.logging.Log` | `tracing::Subscriber` |
//! | `org.apache.commons.logging.LogFactory` | `tracing_subscriber::Registry` |
//! | `org.apache.commons.logging.Log.isDebugEnabled()` | `tracing::enabled!(DEBUG)` |
//! | `org.apache.commons.logging.Log.info()` | `tracing::info!()` |
//! | `org.apache.commons.logging.Log.warn()` | `tracing::warn!()` |
//! | `org.apache.commons.logging.Log.error()` | `tracing::error!()` |

/// 日志级别枚举。
///
/// 对标 Spring `org.apache.commons.logging.Log` 的日志级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 跟踪级别（最详细）
    Trace,
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
}

impl LogLevel {
    /// 获取日志级别的字符串表示。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// 日志记录器 trait。
///
/// 对标 Spring `org.apache.commons.logging.Log`。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::logging::{Logger, LogLevel};
///
/// struct MyLogger;
///
/// impl Logger for MyLogger {
///     fn log(&self, level: LogLevel, message: &str) {
///         println!("[{}] {}", level.as_str(), message);
///     }
/// }
///
/// let logger = MyLogger;
/// logger.info("Hello, world!");
/// ```
pub trait Logger: Send + Sync {
    /// 记录日志。
    fn log(&self, level: LogLevel, message: &str);

    /// 记录跟踪日志。
    fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// 记录调试日志。
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// 记录信息日志。
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// 记录警告日志。
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// 记录错误日志。
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    /// 检查是否启用了指定级别的日志。
    fn is_enabled(&self, _level: LogLevel) -> bool {
        true
    }
}

/// 控制台日志记录器。
///
/// 对标 Spring `org.apache.commons.logging.impl.SimpleLog`。
#[derive(Debug, Clone)]
pub struct ConsoleLogger {
    /// 日志名称
    name: String,
    /// 最低日志级别
    min_level: LogLevel,
}

impl ConsoleLogger {
    /// 创建新的控制台日志记录器。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            min_level: LogLevel::Info,
        }
    }

    /// 设置最低日志级别。
    pub fn set_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        if level >= self.min_level {
            eprintln!("[{}] [{}] {}", level.as_str(), self.name, message);
        }
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }
}

/// 空日志记录器（丢弃所有日志）。
///
/// 对标 Spring `org.apache.commons.logging.impl.NoOpLog`。
#[derive(Debug, Clone)]
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn log(&self, _level: LogLevel, _message: &str) {
        // 丢弃所有日志
    }

    fn is_enabled(&self, _level: LogLevel) -> bool {
        false
    }
}

/// Tracing 日志记录器（feature = "tracing"）。
///
/// 对标 Spring `commons-logging` 集成 `log4j` / `slf4j`。
///
/// 使用 `tracing` crate 作为后端，提供结构化日志记录。
#[cfg(feature = "tracing")]
pub mod tracing_logger {
    use super::{LogLevel, Logger};

    /// Tracing 日志记录器。
    ///
    /// 对标 Spring `org.apache.commons.logging.impl.Slf4jLogFactory`。
    #[derive(Debug, Clone)]
    pub struct TracingLogger {
        name: String,
    }

    impl TracingLogger {
        /// 创建新的 Tracing 日志记录器。
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    impl Logger for TracingLogger {
        fn log(&self, level: LogLevel, message: &str) {
            match level {
                LogLevel::Trace => tracing::trace!("[{}] {}", self.name, message),
                LogLevel::Debug => tracing::debug!("[{}] {}", self.name, message),
                LogLevel::Info => tracing::info!("[{}] {}", self.name, message),
                LogLevel::Warn => tracing::warn!("[{}] {}", self.name, message),
                LogLevel::Error => tracing::error!("[{}] {}", self.name, message),
            }
        }

        fn is_enabled(&self, level: LogLevel) -> bool {
            match level {
                LogLevel::Trace => tracing::enabled!(tracing::Level::TRACE),
                LogLevel::Debug => tracing::enabled!(tracing::Level::DEBUG),
                LogLevel::Info => tracing::enabled!(tracing::Level::INFO),
                LogLevel::Warn => tracing::enabled!(tracing::Level::WARN),
                LogLevel::Error => tracing::enabled!(tracing::Level::ERROR),
            }
        }
    }
}

/// Tracing 日志记录器（feature = "tracing"）。
///
/// 对标 Spring `commons-logging` 集成 `log4j` / `slf4j`。
///
/// 使用 `tracing` crate 作为后端，提供结构化日志记录。
#[cfg(feature = "tracing")]
pub mod tracing_logger {
    use super::{LogLevel, Logger};

    /// Tracing 日志记录器。
    ///
    /// 对标 Spring `org.apache.commons.logging.impl.Slf4jLogFactory`。
    #[derive(Debug, Clone)]
    pub struct TracingLogger {
        name: String,
    }

    impl TracingLogger {
        /// 创建新的 Tracing 日志记录器。
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    impl Logger for TracingLogger {
        fn log(&self, level: LogLevel, message: &str) {
            match level {
                LogLevel::Trace => tracing::trace!("[{}] {}", self.name, message),
                LogLevel::Debug => tracing::debug!("[{}] {}", self.name, message),
                LogLevel::Info => tracing::info!("[{}] {}", self.name, message),
                LogLevel::Warn => tracing::warn!("[{}] {}", self.name, message),
                LogLevel::Error => tracing::error!("[{}] {}", self.name, message),
            }
        }

        fn is_enabled(&self, level: LogLevel) -> bool {
            match level {
                LogLevel::Trace => tracing::enabled!(tracing::Level::TRACE),
                LogLevel::Debug => tracing::enabled!(tracing::Level::DEBUG),
                LogLevel::Info => tracing::enabled!(tracing::Level::INFO),
                LogLevel::Warn => tracing::enabled!(tracing::Level::WARN),
                LogLevel::Error => tracing::enabled!(tracing::Level::ERROR),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn console_logger_default_level_is_info() {
        let logger = ConsoleLogger::new("test");
        assert!(logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
        assert!(!logger.is_enabled(LogLevel::Debug));
        assert!(!logger.is_enabled(LogLevel::Trace));
    }

    #[test]
    fn console_logger_set_level() {
        let mut logger = ConsoleLogger::new("test");
        assert!(logger.is_enabled(LogLevel::Info));
        logger.set_level(LogLevel::Error);
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Error));
    }

    #[test]
    fn console_logger_log_respects_level() {
        let mut logger = ConsoleLogger::new("test");
        logger.set_level(LogLevel::Warn);
        logger.log(LogLevel::Trace, "trace");
        logger.log(LogLevel::Debug, "debug");
        logger.log(LogLevel::Info, "info");
        logger.log(LogLevel::Warn, "warn");
        logger.log(LogLevel::Error, "error");
    }

    #[test]
    fn noop_logger_disables_all() {
        let logger = NoOpLogger;
        assert!(!logger.is_enabled(LogLevel::Trace));
        assert!(!logger.is_enabled(LogLevel::Error));
    }

    #[test]
    fn noop_logger_log_does_nothing() {
        let logger = NoOpLogger;
        logger.log(LogLevel::Trace, "test");
        logger.log(LogLevel::Debug, "test");
        logger.log(LogLevel::Info, "test");
        logger.log(LogLevel::Warn, "test");
        logger.log(LogLevel::Error, "test");
    }

    #[test]
    fn noop_logger_default_methods() {
        let logger = NoOpLogger;
        logger.trace("test");
        logger.debug("test");
        logger.info("test");
        logger.warn("test");
        logger.error("test");
    }
}
