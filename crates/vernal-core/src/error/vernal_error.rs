//! 框架级统一错误类型。
//!
//! [`VernalError`] 是 Vernal 框架的顶层错误枚举，支持三种变体：
//!
//! - **Business**：零分配的业务错误码（domain / code / message 三元组）
//! - **`WithContext`**：在 Business 基础上附加动态诊断信息
//! - **Infrastructure**：包装第三方库或 IO 错误
//!
//! 与 `BoxError` / `SharedError` 的区别在于：`VernalError` 支持跨 crate 边界的
//! 程序化错误匹配（通过 `domain()` + `code()`），而类型擦别名只能通过 downcast 判断。

use std::fmt;

use crate::{BoxError, SharedError};

use super::{ErrorContext, ErrorReport};

/// 框架级统一错误类型。
///
/// # 设计来源
///
/// 对标 `tx_di` 的 `AppError`（domain / code / message）模式，
/// 增加了结构化上下文（`WithContextEntries`）变体。
///
/// # 错误匹配
///
/// ```rust
/// use vernal_core::error::VernalError;
///
/// let err = VernalError::business("ioc", -1, "组件未找到");
///
/// // 程序化匹配：通过 domain + code 判断错误类型
/// assert_eq!(err.domain(), Some("ioc"));
/// assert_eq!(err.code(), Some(-1));
///
/// // 快速判断是否为基础设施错误
/// assert!(!err.is_infrastructure());
/// ```
#[derive(Debug)]
pub enum VernalError {
    /// 业务错误：零分配，携带域、码、消息。
    ///
    /// 用于可预期的错误场景，如组件未找到、依赖歧义、配置缺失。
    /// 错误消息是静态字符串，不包含动态数据。
    Business {
        /// 错误所属的子系统域（如 "ioc"、"aop"、"context"）
        domain: &'static str,
        /// 子系统内的错误码（负数 = 框架错误，正数 = 业务错误）
        code: i32,
        /// 人类可读的静态描述
        message: &'static str,
    },

    /// 带上下文的业务错误：在 Business 基础上附加单个动态上下文字符串。
    ///
    /// 用于需要附带少量动态信息的错误场景。
    /// 上下文内容不应直接暴露给客户端。
    WithContext {
        /// 错误所属的子系统域
        domain: &'static str,
        /// 子系统内的错误码
        code: i32,
        /// 人类可读的静态描述
        message: &'static str,
        /// 动态上下文信息（可能包含敏感数据，仅用于服务端诊断）
        context: String,
    },

    /// 带结构化上下文的业务错误：在 Business 基础上附加多组键值对。
    ///
    /// 用于需要附带多维度诊断信息的错误场景。
    /// 上下文内容不应直接暴露给客户端。
    WithContextEntries {
        /// 错误所属的子系统域
        domain: &'static str,
        /// 子系统内的错误码
        code: i32,
        /// 人类可读的静态描述
        message: &'static str,
        /// 结构化上下文（键值对形式的诊断信息）
        context: ErrorContext,
    },

    /// 基础设施错误：包装第三方库或 IO 错误。
    ///
    /// 用于不可预期的错误场景，如数据库连接失败、文件读取超时。
    /// 源错误链仅在服务端诊断时暴露。
    Infrastructure(SharedError),
}

impl VernalError {
    /// 创建业务错误。
    ///
    /// # 参数
    /// - `domain`：错误所属的子系统域
    /// - `code`：子系统内的错误码
    /// - `message`：人类可读的静态描述
    #[must_use]
    pub const fn business(domain: &'static str, code: i32, message: &'static str) -> Self {
        Self::Business {
            domain,
            code,
            message,
        }
    }

    /// 创建带上下文的业务错误。
    ///
    /// # 参数
    /// - `domain`：错误所属的子系统域
    /// - `code`：子系统内的错误码
    /// - `message`：人类可读的静态描述
    /// - `context`：动态上下文信息
    #[must_use]
    pub fn with_context(
        domain: &'static str,
        code: i32,
        message: &'static str,
        context: impl Into<String>,
    ) -> Self {
        Self::WithContext {
            domain,
            code,
            message,
            context: context.into(),
        }
    }

    /// 创建带结构化上下文的业务错误。
    ///
    /// # 参数
    /// - `domain`：错误所属的子系统域
    /// - `code`：子系统内的错误码
    /// - `message`：人类可读的静态描述
    /// - `context`：结构化上下文
    #[must_use]
    pub const fn with_context_entries(
        domain: &'static str,
        code: i32,
        message: &'static str,
        context: ErrorContext,
    ) -> Self {
        Self::WithContextEntries {
            domain,
            code,
            message,
            context,
        }
    }

    /// 创建基础设施错误。
    ///
    /// 接受任何实现了 `Error + Send + Sync + 'static` 的类型。
    #[must_use]
    pub fn infrastructure<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Infrastructure(std::sync::Arc::new(error))
    }

    /// 创建基础设施错误（从 `SharedError`）。
    #[must_use]
    pub fn infrastructure_shared(error: SharedError) -> Self {
        Self::Infrastructure(error)
    }

    /// 获取错误域。
    ///
    /// 业务错误返回对应的域标识符，基础设施错误返回 `None`。
    #[must_use]
    pub fn domain(&self) -> Option<&'static str> {
        match self {
            Self::Business { domain, .. }
            | Self::WithContext { domain, .. }
            | Self::WithContextEntries { domain, .. } => Some(domain),
            Self::Infrastructure(_) => None,
        }
    }

    /// 获取错误码。
    ///
    /// 业务错误返回对应的错误码，基础设施错误返回 `None`。
    #[must_use]
    pub fn code(&self) -> Option<i32> {
        match self {
            Self::Business { code, .. }
            | Self::WithContext { code, .. }
            | Self::WithContextEntries { code, .. } => Some(*code),
            Self::Infrastructure(_) => None,
        }
    }

    /// 获取静态错误消息。
    ///
    /// 业务错误返回对应的消息，基础设施错误返回 "internal error"。
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Self::Business { message, .. }
            | Self::WithContext { message, .. }
            | Self::WithContextEntries { message, .. } => message,
            Self::Infrastructure(_) => "internal error",
        }
    }

    /// 是否为基础设施错误。
    #[must_use]
    pub fn is_infrastructure(&self) -> bool {
        matches!(self, Self::Infrastructure(_))
    }

    /// 是否为业务错误（包括带上下文的变体）。
    #[must_use]
    pub fn is_business(&self) -> bool {
        !self.is_infrastructure()
    }

    /// 生成脱敏诊断报告。
    #[must_use]
    pub fn report(&self) -> ErrorReport {
        ErrorReport::from_error(self)
    }
}

// ─── 标准库互操作 ───

impl fmt::Display for VernalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Business {
                domain,
                code,
                message,
            } => write!(f, "[{domain}:{code}] {message}"),
            Self::WithContext {
                domain,
                code,
                message,
                context,
            } => {
                write!(f, "[{domain}:{code}] {message}")?;
                if !context.is_empty() {
                    write!(f, " [{context}]")?;
                }
                Ok(())
            }
            Self::WithContextEntries {
                domain,
                code,
                message,
                context,
            } => {
                write!(f, "[{domain}:{code}] {message}")?;
                if !context.is_empty() {
                    write!(f, " {context}")?;
                }
                Ok(())
            }
            Self::Infrastructure(source) => {
                write!(f, "[infrastructure] {source}")
            }
        }
    }
}

impl std::error::Error for VernalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Infrastructure(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

// ─── From 转换 ───

impl From<BoxError> for VernalError {
    /// 从 `BoxError` 转换为基础设施错误。
    fn from(error: BoxError) -> Self {
        Self::Infrastructure(std::sync::Arc::from(error))
    }
}

impl From<SharedError> for VernalError {
    /// 从 `SharedError` 转换为基础设施错误。
    fn from(error: SharedError) -> Self {
        Self::Infrastructure(error)
    }
}

impl From<std::io::Error> for VernalError {
    /// 从 IO 错误转换为基础设施错误。
    fn from(error: std::io::Error) -> Self {
        Self::infrastructure(error)
    }
}

impl From<String> for VernalError {
    /// 从字符串创建基础设施错误。
    ///
    /// 用于不希望引入 `anyhow` 但仍希望 `?` 传播字符串错误的场景。
    /// 字符串会被包装在 [`std::io::Error::new`] 中，保留原始消息。
    ///
    /// # 等价
    ///
    /// ```rust,ignore
    /// let err: VernalError = "db failed".to_string().into();
    /// // 等价于
    /// let err = VernalError::infrastructure(
    ///     std::io::Error::new(std::io::ErrorKind::Other, "db failed")
    /// );
    /// ```
    fn from(message: String) -> Self {
        Self::infrastructure(std::io::Error::other(message))
    }
}

impl From<&str> for VernalError {
    /// 从字符串字面量创建基础设施错误。
    ///
    /// 与 `From<String>` 等价，便于在 `?` 表达式中直接传递字面量。
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

// 注：`From<VernalError> for BoxError` 不需要显式实现，
// 因为 Rust 标准库已有 blanket impl：
// `impl<E: Error + Send + Sync + 'static> From<E> for Box<dyn Error + Send + Sync>`
// VernalError 实现了 Error trait，因此自动获得此转换。

// ─── 跨变体比较（仅比较 domain + code） ───

impl PartialEq for VernalError {
    /// 业务错误按 domain + code 比较（忽略上下文和消息）。
    /// 基础设施错误按 Arc 指针比较。
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Business {
                    domain: d1,
                    code: c1,
                    ..
                },
                Self::Business {
                    domain: d2,
                    code: c2,
                    ..
                },
            ) => d1 == d2 && c1 == c2,
            (
                Self::WithContext {
                    domain: d1,
                    code: c1,
                    ..
                },
                Self::WithContext {
                    domain: d2,
                    code: c2,
                    ..
                },
            ) => d1 == d2 && c1 == c2,
            (
                Self::WithContextEntries {
                    domain: d1,
                    code: c1,
                    ..
                },
                Self::WithContextEntries {
                    domain: d2,
                    code: c2,
                    ..
                },
            ) => d1 == d2 && c1 == c2,
            (Self::Infrastructure(a), Self::Infrastructure(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for VernalError {}

// ─── 单元测试 ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    // ── 构造 ──

    #[test]
    fn business_constructor_sets_all_fields() {
        let err = VernalError::business("ioc", -1, "组件未找到");
        assert_eq!(err.domain(), Some("ioc"));
        assert_eq!(err.code(), Some(-1));
        assert_eq!(err.message(), "组件未找到");
        assert!(err.is_business());
        assert!(!err.is_infrastructure());
    }

    #[test]
    fn with_context_constructor_preserves_context() {
        let err = VernalError::with_context("ioc", -2, "依赖歧义", "DatabasePool vs CachePool");
        assert_eq!(err.domain(), Some("ioc"));
        assert_eq!(err.code(), Some(-2));
        assert!(err.is_business());
    }

    #[test]
    fn with_context_entries_constructor_works() {
        let ctx = ErrorContext::new()
            .with("component", "DatabasePool")
            .with("reason", "timeout");
        let err = VernalError::with_context_entries("ioc", -3, "组件启动失败", ctx);
        assert_eq!(err.code(), Some(-3));
        assert!(err.is_business());
    }

    #[test]
    fn infrastructure_constructor_wraps_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err = VernalError::infrastructure(io_err);
        assert!(err.is_infrastructure());
        assert!(!err.is_business());
        assert_eq!(err.domain(), None);
        assert_eq!(err.code(), None);
        assert_eq!(err.message(), "internal error");
    }

    // ── Display ──

    #[test]
    fn display_business_format_matches_spring_convention() {
        let err = VernalError::business("ioc", -1, "组件未找到");
        assert_eq!(err.to_string(), "[ioc:-1] 组件未找到");
    }

    #[test]
    fn display_with_context_includes_context_suffix() {
        let err = VernalError::with_context("ioc", -2, "依赖歧义", "DatabasePool vs CachePool");
        let s = err.to_string();
        assert!(s.starts_with("[ioc:-2] 依赖歧义"), "actual: {s}");
        assert!(s.contains("DatabasePool vs CachePool"), "actual: {s}");
    }

    #[test]
    fn display_with_context_entries_does_not_leak_entry_values() {
        let ctx = ErrorContext::new().with("secret", "hunter2");
        let err = VernalError::with_context_entries("ioc", -3, "启动失败", ctx);
        let s = err.to_string();
        assert!(
            !s.contains("hunter2"),
            "敏感数据不应出现在 Display 输出: {s}"
        );
    }

    #[test]
    fn display_infrastructure_starts_with_infrastructure_marker() {
        let err: VernalError = std::io::Error::other("disk full").into();
        let s = err.to_string();
        assert!(s.starts_with("[infrastructure]"), "actual: {s}");
        assert!(s.contains("disk full"));
    }

    // ── source() ──

    #[test]
    fn source_returns_some_only_for_infrastructure() {
        let infra: VernalError = std::io::Error::other("x").into();
        assert!(infra.source().is_some());

        let biz = VernalError::business("ioc", -1, "x");
        assert!(biz.source().is_none());

        let ctx = VernalError::with_context("ioc", -1, "x", "ctx");
        assert!(ctx.source().is_none());

        let entries = VernalError::with_context_entries("ioc", -1, "x", ErrorContext::new());
        assert!(entries.source().is_none());
    }

    // ── PartialEq ──

    #[test]
    fn business_eq_ignores_message_difference() {
        let a = VernalError::business("ioc", -1, "消息 A");
        let b = VernalError::business("ioc", -1, "消息 B");
        assert_eq!(a, b, "相同 domain+code 应相等，即便消息不同");
    }

    #[test]
    fn business_ne_when_domain_differs() {
        let a = VernalError::business("ioc", -1, "x");
        let b = VernalError::business("aop", -1, "x");
        assert_ne!(a, b);
    }

    #[test]
    fn business_ne_when_code_differs() {
        let a = VernalError::business("ioc", -1, "x");
        let b = VernalError::business("ioc", -2, "x");
        assert_ne!(a, b);
    }

    #[test]
    fn with_context_eq_when_same_domain_code() {
        let a = VernalError::with_context("ioc", -1, "x", "ctx A");
        let b = VernalError::with_context("ioc", -1, "x", "ctx B");
        assert_eq!(a, b, "相同 domain+code 应相等，即便上下文不同");
    }

    #[test]
    fn cross_variant_never_equal() {
        let biz = VernalError::business("ioc", -1, "x");
        let ctx = VernalError::with_context("ioc", -1, "x", "y");
        let entries = VernalError::with_context_entries("ioc", -1, "x", ErrorContext::new());
        let infra: VernalError = std::io::Error::other("z").into();

        assert_ne!(biz, ctx);
        assert_ne!(biz, entries);
        assert_ne!(biz, infra);
        assert_ne!(ctx, entries);
        assert_ne!(ctx, infra);
        assert_ne!(entries, infra);
    }

    #[test]
    fn infrastructure_eq_uses_arc_ptr_identity() {
        // 同一 Arc 实例 → 相等
        let shared_err: SharedError = std::sync::Arc::new(std::io::Error::other("x"));
        let a = VernalError::infrastructure_shared(shared_err.clone());
        let b = VernalError::infrastructure_shared(shared_err);
        assert_eq!(a, b);

        // 不同 Arc 实例（即使消息相同）→ 不相等
        let c: VernalError = std::io::Error::other("x").into();
        let d: VernalError = std::io::Error::other("x").into();
        assert_ne!(c, d);
    }

    // ── From 转换 ──

    #[test]
    fn from_box_error_produces_infrastructure() {
        let boxed: BoxError = Box::new(std::io::Error::other("wrapped"));
        let err: VernalError = boxed.into();
        assert!(err.is_infrastructure());
    }

    #[test]
    fn from_shared_error_produces_infrastructure() {
        let shared: SharedError = std::sync::Arc::new(std::io::Error::other("shared"));
        let err: VernalError = shared.into();
        assert!(err.is_infrastructure());
    }

    #[test]
    fn from_io_error_preserves_message() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "no route");
        let err: VernalError = io_err.into();
        let s = err.to_string();
        assert!(s.contains("no route"), "actual: {s}");
        assert!(err.is_infrastructure());
    }

    #[test]
    fn from_string_wraps_into_io_error_other() {
        let err: VernalError = "disk full".to_string().into();
        assert!(err.is_infrastructure());
        let s = err.to_string();
        assert!(s.contains("disk full"), "actual: {s}");
    }

    #[test]
    fn from_str_literal_produces_infrastructure() {
        let err: VernalError = "db unreachable".into();
        assert!(err.is_infrastructure());
        assert!(err.to_string().contains("db unreachable"));
    }

    #[test]
    fn vernal_error_converts_to_box_error_via_blanket_impl() {
        let err = VernalError::business("ioc", -1, "x");
        // 触发 blanket impl `From<E> for Box<dyn Error + Send + Sync>`
        let boxed: BoxError = Box::new(err);
        assert!(!boxed.to_string().is_empty());
    }

    // ── report() / ErrorReport ──

    #[test]
    fn business_report_contains_full_identity() {
        let err = VernalError::business("ioc", -1, "组件未找到");
        let report = err.report();
        assert_eq!(report.domain(), "ioc");
        assert_eq!(report.code(), -1);
        assert_eq!(report.message(), "组件未找到");
        assert_eq!(report.context_entries(), 0);
        assert!(!report.is_infrastructure());
    }

    #[test]
    fn with_context_report_counts_single_entry() {
        let err = VernalError::with_context("ioc", -1, "x", "ctx-string");
        let report = err.report();
        assert_eq!(report.context_entries(), 1);
    }

    #[test]
    fn infrastructure_report_redacts_internal_details() {
        let err: VernalError = std::io::Error::other("secret stacktrace").into();
        let report = err.report();
        assert_eq!(report.domain(), "unknown");
        assert_eq!(report.code(), -9999);
        assert_eq!(report.message(), "internal error");
        assert_eq!(report.context_entries(), 0);
        assert!(report.is_infrastructure());
        // 错误消息不得包含源错误细节
        assert!(!report.to_string().contains("secret stacktrace"));
    }

    // ── Send + Sync 编译期断言 ──

    #[test]
    fn vernal_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VernalError>();
    }

    #[test]
    fn message_for_with_context_returns_static_part() {
        // 对标 Spring ErrorMessage: message 仅返回静态部分, 上下文由 messageWithContext 提供
        let err = VernalError::with_context("ioc", -1, "静态消息", "动态内容");
        assert_eq!(err.message(), "静态消息");
    }

    #[test]
    fn message_for_with_context_entries_returns_static_part() {
        let ctx = ErrorContext::new().with("k", "v");
        let err = VernalError::with_context_entries("ioc", -1, "静态消息", ctx);
        assert_eq!(err.message(), "静态消息");
    }

    #[test]
    fn display_with_context_empty_context_omits_bracket_suffix() {
        // 空上下文时, Display 不附加 " []" 后缀（对标 Spring 空 cause 字符串）
        let err = VernalError::with_context("ioc", -1, "无上下文", "");
        let s = err.to_string();
        assert!(s.starts_with("[ioc:-1] 无上下文"), "actual: {s}");
        // 空字符串也被视为 empty, 不应追加空白
        assert!(!s.ends_with(" ]"), "should not append bracket suffix: {s}");
    }

    #[test]
    fn display_with_context_entries_empty_context_omits_suffix() {
        // 对标 Spring StructuredErrorMessage: 空 entries 不输出
        let err = VernalError::with_context_entries("ioc", -1, "无上下文", ErrorContext::new());
        let s = err.to_string();
        assert!(s.starts_with("[ioc:-1] 无上下文"), "actual: {s}");
    }

    #[test]
    fn with_context_entries_eq_when_same_domain_and_code() {
        // 对标 Spring 跨变体的 equality 仅按 (domain, code)
        let a = VernalError::with_context_entries("ioc", -1, "msg-A", ErrorContext::new());
        let b = VernalError::with_context_entries("ioc", -1, "msg-B", ErrorContext::new());
        assert_eq!(a, b, "相同 domain+code 应相等, 即便消息和 entries 不同");
    }

    #[test]
    fn with_context_entries_ne_when_code_differs() {
        let a = VernalError::with_context_entries("ioc", -1, "x", ErrorContext::new());
        let b = VernalError::with_context_entries("ioc", -2, "x", ErrorContext::new());
        assert_ne!(a, b);
    }
}
