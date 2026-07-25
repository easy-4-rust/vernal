//! Axum 提取器拒绝对象。

use std::{error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use vernal_beans::ResolveError;

/// 将 Vernal 基础设施失败映射为稳定且不泄露内部细节的 Axum 响应。
#[derive(Debug)]
pub enum AxumRejection {
    /// 请求没有经过 `VernalLayer`。
    MissingContext,
    /// 请求没有经过 `AxumRequestScope` 中间件。
    MissingRequestScope,
    /// 请求没有经过严格 AOP/Context Propagation Layer。
    MissingRequestContext,
    /// `IoC` 容器无法解析目标组件。
    ComponentResolution {
        /// 仅供日志和错误链检查的原始解析错误。
        source: ResolveError,
    },
}

impl AxumRejection {
    /// 创建组件解析拒绝。
    #[must_use]
    pub const fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution { source }
    }

    /// 返回稳定 HTTP 状态码。
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// 返回可安全发送给客户端的消息。
    #[must_use]
    pub const fn safe_message(&self) -> &'static str {
        match self {
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::MissingRequestContext => "Vernal request context is unavailable",
            Self::ComponentResolution { .. } => "Vernal component resolution failed",
        }
    }
}

impl IntoResponse for AxumRejection {
    fn into_response(self) -> Response {
        (self.status(), self.safe_message()).into_response()
    }
}

impl fmt::Display for AxumRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Axum request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Axum request has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Axum request has no Vernal request context")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Axum component resolution failed: {source}")
            }
        }
    }
}

impl Error for AxumRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source),
            _ => None,
        }
    }
}
