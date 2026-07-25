//! 框架级统一错误类型。
//!
//! [`VernalError`] 是 Vernal 框架的顶层错误枚举，支持三种变体：
//!
//! - **Business**：零分配的业务错误码（domain / code / message 三元组）
//! - **WithContext**：在 Business 基础上附加动态诊断信息
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
/// 对标 tx_di 的 `AppError`（domain / code / message）模式，
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
    pub const fn business(
        domain: &'static str,
        code: i32,
        message: &'static str,
    ) -> Self {
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

    /// 创建基础设施错误（从 SharedError）。
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
    /// 从 BoxError 转换为基础设施错误。
    fn from(error: BoxError) -> Self {
        Self::Infrastructure(std::sync::Arc::from(error))
    }
}

impl From<SharedError> for VernalError {
    /// 从 SharedError 转换为基础设施错误。
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
