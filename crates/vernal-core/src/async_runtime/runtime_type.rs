//! 运行时类型枚举。
//!
//! 对标 Spring `org.springframework.core.task.TaskExecutor` 派发策略。

/// 运行时类型枚举。
///
/// 对应 Java: 多种 TaskExecutor 抽象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    /// tokio 运行时
    Tokio,
    /// async-std 运行时
    AsyncStd,
    /// 当前线程（同步）
    CurrentThread,
}

impl RuntimeType {
    /// 返回运行时名称。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tokio => "tokio",
            Self::AsyncStd => "async-std",
            Self::CurrentThread => "current-thread",
        }
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_name() {
        assert_eq!(RuntimeType::Tokio.as_str(), "tokio");
        assert_eq!(RuntimeType::AsyncStd.as_str(), "async-std");
        assert_eq!(RuntimeType::CurrentThread.as_str(), "current-thread");
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", RuntimeType::Tokio), "tokio");
    }

    #[test]
    fn variants_distinct() {
        assert_ne!(RuntimeType::Tokio, RuntimeType::AsyncStd);
        assert_ne!(RuntimeType::AsyncStd, RuntimeType::CurrentThread);
    }
}
