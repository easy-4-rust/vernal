//! Ntex 提取器与中间件拒绝对象。

use std::{error::Error, fmt};

use ntex::{
    http::StatusCode,
    web::{ErrorRenderer, HttpRequest, HttpResponse, WebResponseError},
};
use vernal_beans::ResolveError;
use vernal_web::ScopeError;

use crate::NtexAopError;

/// 将 Vernal 基础设施失败映射为稳定、脱敏的 Ntex 响应。
#[derive(Debug)]
pub enum NtexRejection {
    /// 当前请求没有应用上下文。
    MissingContext,
    /// 当前请求没有请求作用域。
    MissingRequestScope,
    /// 当前请求没有严格 Local-AOP 创建的请求上下文。
    MissingRequestContext,
    /// `IoC` 容器无法解析目标组件。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
    /// 请求 Scope 关闭失败。
    ScopeClose {
        /// 原始关闭错误，仅供服务端错误链使用。
        source: ScopeError,
    },
    /// 严格 Local-AOP 调用失败且没有可用于直接构造响应的请求信封。
    Aop {
        /// 原始 AOP 错误，仅供服务端错误链使用。
        source: NtexAopError,
    },
}

impl NtexRejection {
    /// 创建组件解析拒绝。
    #[must_use]
    pub fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution {
            source: Box::new(source),
        }
    }

    /// 创建请求 Scope 关闭拒绝。
    #[must_use]
    pub const fn scope_close(source: ScopeError) -> Self {
        Self::ScopeClose { source }
    }

    /// 创建严格 Local-AOP 拒绝。
    #[must_use]
    pub const fn aop(source: NtexAopError) -> Self {
        Self::Aop { source }
    }

    /// 返回允许发送给客户端的稳定错误消息。
    #[must_use]
    pub fn safe_message(&self) -> &str {
        match self {
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::MissingRequestContext => "Vernal request context is unavailable",
            Self::ComponentResolution { .. } => "Vernal component resolution failed",
            Self::ScopeClose { .. } => "Vernal request scope cleanup failed",
            Self::Aop { source } => source.safe_message(),
        }
    }
}

impl fmt::Display for NtexRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Ntex request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Ntex request has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Ntex request has no Vernal request context")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Ntex component resolution failed: {source}")
            }
            Self::ScopeClose { source } => {
                write!(formatter, "Ntex request scope close failed: {source}")
            }
            Self::Aop { source } => write!(formatter, "Ntex AOP request failed: {source}"),
        }
    }
}

impl Error for NtexRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            Self::ScopeClose { source } => Some(source),
            Self::Aop { source } => Some(source),
            _ => None,
        }
    }
}

impl<Err> WebResponseError<Err> for NtexRejection
where
    Err: ErrorRenderer,
{
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Aop { source } => source.status(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _request: &HttpRequest) -> HttpResponse {
        // `Display` 包含服务端诊断细节；客户端只能收到固定的安全文本。
        match self {
            Self::Aop { source } => source.response(),
            _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body(self.safe_message().to_owned()),
        }
    }
}
