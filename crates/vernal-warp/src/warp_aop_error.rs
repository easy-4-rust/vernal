//! Warp 严格 AOP 错误映射对象。

use std::{convert::Infallible, error::Error, fmt};

use vernal_aop::InvocationError;
use vernal_tower::AopServiceError;
use vernal_web::WebFailure;
use warp::{
    Reply,
    http::StatusCode,
    reply::{Response, with_status},
};

/// 将 Tower AOP 的结构化失败转换为稳定、脱敏的 Warp 原生响应。
#[derive(Debug)]
pub struct WarpAopError {
    source: AopServiceError<Infallible>,
}

impl WarpAopError {
    /// 包装 Warp Service 边界返回的严格 AOP 错误。
    #[must_use]
    pub const fn new(source: AopServiceError<Infallible>) -> Self {
        Self { source }
    }

    /// 返回安全 HTTP 状态码。
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

    /// 返回允许发送给客户端的脱敏消息。
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
            AopServiceError::MissingRouteMetadata => "Warp route pattern is unavailable",
            _ => "Vernal AOP invocation failed",
        }
    }

    /// 构造 Warp 公共 Reply 边界的原生响应。
    #[must_use]
    pub fn response(&self) -> Response {
        with_status(self.safe_message().to_owned(), self.status()).into_response()
    }

    /// 从目标调用失败中读取框架中立 Web 错误。
    fn web_failure(&self) -> Option<&WebFailure> {
        match &self.source {
            AopServiceError::Invocation(InvocationError::Target { source }) => {
                source.downcast_ref::<WebFailure>()
            }
            _ => None,
        }
    }
}

impl fmt::Display for WarpAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Warp AOP request failed: {}", self.source)
    }
}

impl Error for WarpAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
