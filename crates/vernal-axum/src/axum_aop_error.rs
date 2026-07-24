//! Axum AOP 错误映射对象。

use std::{convert::Infallible, error::Error, fmt};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use vernal_aop::InvocationError;
use vernal_tower::AopServiceError;
use vernal_web::WebFailure;

/// 将严格 Tower AOP 失败转换为稳定、脱敏的 Axum 响应。
#[derive(Debug)]
pub struct AxumAopError {
    source: AopServiceError<Infallible>,
}

impl AxumAopError {
    /// 包装 AOP Service 返回的结构化错误。
    #[must_use]
    pub const fn new(source: AopServiceError<Infallible>) -> Self {
        Self { source }
    }

    /// 返回协议状态码。
    #[must_use]
    pub fn status(&self) -> StatusCode {
        if let Some(failure) = self.web_failure() {
            return StatusCode::from_u16(failure.problem().status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        }
        match &self.source {
            AopServiceError::Invocation(InvocationError::Cancelled) => StatusCode::REQUEST_TIMEOUT,
            AopServiceError::Invocation(InvocationError::DeadlineExceeded) => {
                StatusCode::GATEWAY_TIMEOUT
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 返回可以发送给客户端的脱敏消息。
    #[must_use]
    pub fn safe_message(&self) -> &str {
        if let Some(failure) = self.web_failure() {
            return failure
                .problem()
                .detail()
                .unwrap_or_else(|| failure.problem().title());
        }
        match &self.source {
            AopServiceError::Invocation(InvocationError::Cancelled) => "Request was cancelled",
            AopServiceError::Invocation(InvocationError::DeadlineExceeded) => {
                "Request deadline exceeded"
            }
            AopServiceError::MissingApplicationContext => {
                "Vernal application context is unavailable"
            }
            AopServiceError::MissingRequestScope => "Vernal request scope is unavailable",
            AopServiceError::MissingRouteMetadata => "Vernal route metadata is unavailable",
            _ => "Vernal AOP invocation failed",
        }
    }

    /// 从调用链目标错误中读取框架中立 Web 失败。
    fn web_failure(&self) -> Option<&WebFailure> {
        match &self.source {
            AopServiceError::Invocation(InvocationError::Target { source }) => {
                source.downcast_ref::<WebFailure>()
            }
            _ => None,
        }
    }
}

impl IntoResponse for AxumAopError {
    fn into_response(self) -> Response {
        (self.status(), self.safe_message().to_owned()).into_response()
    }
}

impl fmt::Display for AxumAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Axum AOP request failed: {}", self.source)
    }
}

impl Error for AxumAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
