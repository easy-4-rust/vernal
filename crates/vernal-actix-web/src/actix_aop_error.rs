//! Actix Local-AOP 错误映射对象。

use std::{error::Error, fmt};

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use vernal_aop::LocalInvocationError;
use vernal_web::WebFailure;

use crate::ActixRequestSnapshotError;

/// 将 Actix 严格 Local-AOP 失败转换为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum ActixAopError {
    /// 严格中间件无法获得匹配后的低基数路由模板。
    MissingRouteMetadata,
    /// Actix HTTP 0.2 请求元数据无法转换到 Vernal HTTP 1.x 快照。
    RequestSnapshot {
        /// 原始字段转换错误。
        source: ActixRequestSnapshotError,
    },
    /// Local-AOP 调用链、取消、截止时间或目标执行失败。
    Invocation {
        /// 原始结构化本地调用错误。
        source: LocalInvocationError,
    },
    /// Local-AOP 目标被重复推进，原生请求已经被消费。
    RequestAlreadyTaken,
}

impl ActixAopError {
    /// 包装请求快照转换错误。
    #[must_use]
    pub const fn request_snapshot(source: ActixRequestSnapshotError) -> Self {
        Self::RequestSnapshot { source }
    }

    /// 包装 Local-AOP 调用错误。
    #[must_use]
    pub const fn invocation(source: LocalInvocationError) -> Self {
        Self::Invocation { source }
    }

    /// 返回安全 HTTP 状态码。
    #[must_use]
    pub fn status(&self) -> StatusCode {
        if let Some(failure) = self.web_failure() {
            return StatusCode::from_u16(failure.problem().status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        }
        match self {
            Self::Invocation {
                source: LocalInvocationError::Cancelled,
            } => StatusCode::REQUEST_TIMEOUT,
            Self::Invocation {
                source: LocalInvocationError::DeadlineExceeded,
            } => StatusCode::GATEWAY_TIMEOUT,
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
        match self {
            Self::MissingRouteMetadata => "Actix matched route metadata is unavailable",
            Self::RequestSnapshot { .. } => "Actix request metadata conversion failed",
            Self::Invocation {
                source: LocalInvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: LocalInvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal Local-AOP invocation failed",
            Self::RequestAlreadyTaken => "Actix request was already consumed",
        }
    }

    /// 从目标错误中恢复框架中立 Web 失败。
    fn web_failure(&self) -> Option<&WebFailure> {
        match self {
            Self::Invocation {
                source: LocalInvocationError::Target { source },
            } => source.downcast_ref::<WebFailure>(),
            _ => None,
        }
    }
}

impl fmt::Display for ActixAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRouteMetadata => {
                formatter.write_str("Actix request has no matched route metadata")
            }
            Self::RequestSnapshot { source } => {
                write!(formatter, "Actix request snapshot failed: {source}")
            }
            Self::Invocation { source } => {
                write!(formatter, "Actix Local-AOP invocation failed: {source}")
            }
            Self::RequestAlreadyTaken => {
                formatter.write_str("Actix Local-AOP target consumed the request more than once")
            }
        }
    }
}

impl Error for ActixAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RequestSnapshot { source } => Some(source),
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}

impl ResponseError for ActixAopError {
    fn status_code(&self) -> StatusCode {
        ActixAopError::status(self)
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status()).body(self.safe_message().to_owned())
    }
}
