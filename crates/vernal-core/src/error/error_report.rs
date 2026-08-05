//! 脱敏诊断报告。
//!
//! 当应用启动或运行失败时，[`ErrorReport`] 提供一份安全的诊断快照，
//! 包含错误域、码、消息和上下文条目数量，但**不包含**上下文的具体值。
//!
//! 这份报告可以安全地写入日志、返回给客户端、或存储到监控系统。

use std::fmt;

use super::{ErrorKind, VernalError};

/// 脱敏诊断报告。
///
/// 从 [`VernalError`] 提取诊断信息，同时过滤掉可能包含敏感数据的动态内容。
///
/// # 安全性
///
/// - 错误消息（message）是静态字符串，不包含用户数据
/// - 上下文（context）仅暴露条目数量，不暴露具体键值
/// - 基础设施错误仅暴露 "internal error"，不暴露源错误链
///
/// # 示例
///
/// ```rust
/// use vernal_core::error::{VernalError, ErrorReport};
///
/// let error = VernalError::business("ioc", -1, "组件未找到");
/// let report = ErrorReport::from_error(&error);
///
/// assert_eq!(report.domain(), "ioc");
/// assert_eq!(report.code(), -1);
/// assert_eq!(report.message(), "组件未找到");
/// assert_eq!(report.context_entries(), 0);
/// ```
#[derive(Debug, Clone)]
pub struct ErrorReport {
    /// 错误域
    domain: &'static str,
    /// 错误码
    code: i32,
    /// 静态错误消息
    message: &'static str,
    /// 上下文条目数量（不暴露具体内容）
    context_entries: usize,
    /// 错误分类
    kind: ErrorKind,
}

impl ErrorReport {
    /// 从 `VernalError` 创建脱敏报告。
    #[must_use]
    pub fn from_error(error: &VernalError) -> Self {
        match error {
            VernalError::Business {
                domain,
                code,
                message,
            } => Self {
                domain,
                code: *code,
                message,
                context_entries: 0,
                kind: ErrorKind::Business,
            },
            VernalError::WithContext {
                domain,
                code,
                message,
                context,
            } => Self {
                domain,
                code: *code,
                message,
                // 上下文是单个字符串，计为 1 个条目
                context_entries: usize::from(!context.is_empty()),
                kind: ErrorKind::Business,
            },
            VernalError::WithContextEntries {
                domain,
                code,
                message,
                context,
            } => Self {
                domain,
                code: *code,
                message,
                context_entries: context.len(),
                kind: ErrorKind::Business,
            },
            VernalError::Infrastructure(_) => Self {
                domain: "unknown",
                code: -9999,
                message: "internal error",
                context_entries: 0,
                kind: ErrorKind::Infrastructure,
            },
        }
    }

    /// 从错误码 trait 对象创建报告。
    #[must_use]
    pub fn from_error_code<E: super::ErrorCode>(error: &E) -> Self {
        Self {
            domain: error.domain(),
            code: error.code(),
            message: error.message(),
            context_entries: 0,
            kind: ErrorKind::Business,
        }
    }

    /// 错误域。
    #[must_use]
    pub fn domain(&self) -> &'static str {
        self.domain
    }

    /// 错误码。
    #[must_use]
    pub fn code(&self) -> i32 {
        self.code
    }

    /// 静态错误消息。
    #[must_use]
    pub fn message(&self) -> &'static str {
        self.message
    }

    /// 上下文条目数量。
    #[must_use]
    pub fn context_entries(&self) -> usize {
        self.context_entries
    }

    /// 错误分类。
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// 是否为基础设施错误。
    #[must_use]
    pub fn is_infrastructure(&self) -> bool {
        self.kind == ErrorKind::Infrastructure
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{domain}:{code}] {message}",
            domain = self.domain,
            code = self.code,
            message = self.message,
        )?;
        if self.context_entries > 0 {
            write!(f, " ({} diagnostic entries)", self.context_entries)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorContext;

    #[test]
    fn from_business_error() {
        let err = VernalError::business("ioc", -1, "not found");
        let report = ErrorReport::from_error(&err);
        assert_eq!(report.domain(), "ioc");
        assert_eq!(report.code(), -1);
        assert_eq!(report.message(), "not found");
        assert_eq!(report.context_entries(), 0);
        assert!(!report.is_infrastructure());
    }

    #[test]
    fn from_with_context_error() {
        let err = VernalError::with_context("aop", -2, "no advisor", "id=42");
        let report = ErrorReport::from_error(&err);
        assert_eq!(report.domain(), "aop");
        assert_eq!(report.context_entries(), 1);
    }

    #[test]
    fn from_infrastructure_error() {
        let io_err = std::io::Error::other("fail");
        let err = VernalError::infrastructure(io_err);
        let report = ErrorReport::from_error(&err);
        assert!(report.is_infrastructure());
        assert_eq!(report.message(), "internal error");
    }

    #[test]
    fn display_format() {
        let err = VernalError::business("ioc", -1, "not found");
        let report = ErrorReport::from_error(&err);
        let s = report.to_string();
        assert!(s.contains("ioc"));
        assert!(s.contains("-1"));
        assert!(s.contains("not found"));
    }

    #[test]
    fn display_with_context_entries() {
        let ctx = ErrorContext::new().with("k", "v");
        let err = VernalError::with_context_entries("ioc", -1, "err", ctx);
        let report = ErrorReport::from_error(&err);
        let s = report.to_string();
        assert!(s.contains("1 diagnostic entries"));
    }

    #[test]
    fn from_error_code_has_correct_kind() {
        use crate::error::ErrorCode;
        struct TestErrorCode;
        impl ErrorCode for TestErrorCode {
            fn domain(&self) -> &'static str {
                "TEST"
            }
            fn code(&self) -> i32 {
                42
            }
            fn message(&self) -> &'static str {
                "test error"
            }
        }
        let report = ErrorReport::from_error_code(&TestErrorCode);
        assert_eq!(report.kind(), ErrorKind::Business);
    }

    #[test]
    fn display_without_context_entries() {
        // 对标 Spring: 无上下文时 Display 不包含 "diagnostic entries"
        let err = VernalError::business("web", -1, "bad request");
        let report = ErrorReport::from_error(&err);
        let s = report.to_string();
        assert!(s.contains("[web:-1]"));
        assert!(s.contains("bad request"));
        assert!(!s.contains("diagnostic entries"));
    }

    #[test]
    fn display_infrastructure_error() {
        let io_err = std::io::Error::other("timeout");
        let err = VernalError::infrastructure(io_err);
        let report = ErrorReport::from_error(&err);
        let s = report.to_string();
        assert!(s.contains("[unknown:-9999]"));
        assert!(s.contains("internal error"));
    }

    #[test]
    fn from_with_context_empty_string() {
        // 对标 Spring: 空上下文字符串不计入 context_entries
        let err = VernalError::with_context("aop", -2, "err", "");
        let report = ErrorReport::from_error(&err);
        assert_eq!(report.context_entries(), 0);
    }

    #[test]
    fn display_write_macro_path_with_entries() {
        // 对标 Spring: Display 格式化包含上下文条目的报告
        // 覆盖行 157: Display write! 宏的完整执行路径
        let ctx = ErrorContext::new().with("k1", "v1").with("k2", "v2");
        let err = VernalError::with_context_entries("domain", 42, "msg", ctx);
        let report = ErrorReport::from_error(&err);
        let s = format!("{report}");
        assert!(s.contains("domain"));
        assert!(s.contains("42"));
        assert!(s.contains("msg"));
        assert!(s.contains("2 diagnostic entries"));
    }

    #[test]
    fn error_report_clone_and_debug() {
        // 对标 Spring: ErrorReport 应支持 Clone 和 Debug
        let err = VernalError::business("d", 1, "m");
        let report = ErrorReport::from_error(&err);
        let cloned = report.clone();
        assert_eq!(cloned.domain(), "d");
        let _debug = format!("{report:?}");
    }
}
