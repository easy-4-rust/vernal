//! 组合日志记录器。
//!
//! 对标 Spring `org.springframework.core.log.CompositeLog`。

use crate::logging::{LogLevel, Logger};

/// 组合日志记录器。
///
/// 对应 Java: org.springframework.core.log.CompositeLog
///
/// Spring 语义：把消息广播给多个底层日志器（如控制台 + 追踪）。
pub struct CompositeLog {
    loggers: Vec<Box<dyn Logger>>,
}

impl CompositeLog {
    /// 创建组合日志器。
    #[must_use]
    pub fn new(loggers: Vec<Box<dyn Logger>>) -> Self {
        Self { loggers }
    }

    /// 追加日志器。
    pub fn add_logger(&mut self, logger: Box<dyn Logger>) {
        self.loggers.push(logger);
    }
}

impl Logger for CompositeLog {
    fn log(&self, level: LogLevel, message: &str) {
        for logger in &self.loggers {
            logger.log(level, message);
        }
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        self.loggers.iter().any(|logger| logger.is_enabled(level))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingLogger {
        count: Arc<AtomicUsize>,
    }

    impl Logger for CountingLogger {
        fn log(&self, _level: LogLevel, _message: &str) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn broadcasts_to_all_loggers() {
        // A 类（合同对齐）：对标 Spring 广播语义
        let first = Arc::new(AtomicUsize::new(0));
        let second = Arc::new(AtomicUsize::new(0));
        let mut composite = CompositeLog::new(vec![
            Box::new(CountingLogger { count: first.clone() }),
            Box::new(CountingLogger { count: second.clone() }),
        ]);
        composite.log(LogLevel::Info, "hello");
        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn enabled_if_any_logger_enabled() {
        // B 类（边界行为）
        let mut composite = CompositeLog::new(Vec::new());
        composite.add_logger(Box::new(CountingLogger {
            count: Arc::new(AtomicUsize::new(0)),
        }));
        assert!(composite.is_enabled(LogLevel::Info));
    }
}
