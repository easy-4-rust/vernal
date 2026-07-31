//! 日志级别。
//!
//! 对标 Spring `org.springframework.util.logging.LogLevel`（commons-logging 适配层）。
//! 提供 trace/debug/info/warn/error 五级严重性顺序。

/// 日志级别枚举。
///
/// 对应 Java: org.springframework.util.logging.LogLevel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// trace 级别（最低）
    Trace,
    /// debug 级别
    Debug,
    /// info 级别
    Info,
    /// warn 级别
    Warn,
    /// error 级别（最高）
    Error,
}

impl LogLevel {
    /// 返回 Spring 兼容的字符串表示。
    ///
    /// 对应 Java: `LogLevel#name`
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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
    fn display_matches_as_str_for_all_variants() {
        // 对标 Spring LogLevel: Display 输出与 as_str 一致
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn log_level_copy_and_eq() {
        let a = LogLevel::Info;
        let b = a; // Copy
        assert_eq!(a, b);
        assert_ne!(LogLevel::Info, LogLevel::Warn);
    }
}
