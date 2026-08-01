//! 日志访问器。
//!
//! 对标 Spring `org.springframework.core.log.LogAccessor`。

use crate::logging::Logger;

/// 日志访问器。
///
/// 对应 Java: org.springframework.core.log.LogAccessor
///
/// Spring 语义：围绕 `Log` 的便捷访问器，提供 `debug`/`info`/`warn`/`error`
/// 及消息构造回调形态。
pub struct LogAccessor {
    logger: std::sync::Arc<dyn Logger>,
}

impl LogAccessor {
    /// 从日志器创建访问器。
    #[must_use]
    pub fn new(logger: std::sync::Arc<dyn Logger>) -> Self {
        Self { logger }
    }

    /// 返回底层日志器。
    #[must_use]
    pub fn logger(&self) -> &dyn Logger {
        self.logger.as_ref()
    }

    /// 输出 debug 消息。
    pub fn debug(&self, message: &str) {
        self.logger.debug(message);
    }

    /// 输出 info 消息。
    pub fn info(&self, message: &str) {
        self.logger.info(message);
    }

    /// 输出 warn 消息。
    pub fn warn(&self, message: &str) {
        self.logger.warn(message);
    }

    /// 输出 error 消息。
    pub fn error(&self, message: &str) {
        self.logger.error(message);
    }

    /// debug 是否启用（对标 Spring `isDebugEnabled`）。
    #[must_use]
    pub fn is_debug_enabled(&self) -> bool {
        self.logger.is_enabled(crate::logging::LogLevel::Debug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{ConsoleLogger, LogLevel};

    #[test]
    fn delegates_to_logger() {
        // A 类（合同对齐）：对标 Spring LogAccessor 委托
        let mut console = ConsoleLogger::new("test");
        console.set_level(LogLevel::Warn);
        let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(console);
        let accessor = LogAccessor::new(logger);
        accessor.debug("skip me");
        accessor.info("skip me too");
        accessor.warn("shown");
        accessor.error("shown too");
        assert!(!accessor.is_debug_enabled());
        assert!(accessor.logger().is_enabled(crate::logging::LogLevel::Warn));
    }
}
