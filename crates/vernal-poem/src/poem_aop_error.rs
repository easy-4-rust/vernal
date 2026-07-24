//! Poem 严格 AOP 错误映射对象。

use std::{error::Error, fmt};

use poem::{Body, Response, error::ResponseError, http::StatusCode};
use vernal_aop::InvocationError;
use vernal_web::WebFailure;

/// 将 Poem 严格 AOP 失败转换为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum PoemAopError {
    /// 严格中间件没有位于已经完成路由匹配的 Endpoint 内层。
    MissingRouteMetadata,
    /// AOP 调用链、取消、截止时间或目标调用失败。
    Invocation {
        /// 原始结构化调用错误，仅供服务端错误链和诊断使用。
        source: InvocationError,
    },
    /// AOP 目标被错误地推进多次，原生请求已经被消费。
    RequestAlreadyTaken,
    /// 原生响应信封已经被消费。
    ResponseAlreadyTaken,
}

impl PoemAopError {
    /// 包装 AOP 调用错误。
    #[must_use]
    pub const fn invocation(source: InvocationError) -> Self {
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
                source: InvocationError::Cancelled,
            } => StatusCode::REQUEST_TIMEOUT,
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
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
            Self::MissingRouteMetadata => "Poem matched route metadata is unavailable",
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal AOP invocation failed",
            Self::RequestAlreadyTaken => "Poem request was already consumed",
            Self::ResponseAlreadyTaken => "Poem response was already consumed",
        }
    }

    /// 从调用目标错误中恢复框架中立 Web 失败。
    fn web_failure(&self) -> Option<&WebFailure> {
        match self {
            Self::Invocation {
                source: InvocationError::Target { source },
            } => source.downcast_ref::<WebFailure>(),
            _ => None,
        }
    }
}

impl fmt::Display for PoemAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRouteMetadata => {
                formatter.write_str("Poem request has no matched route metadata")
            }
            Self::Invocation { source } => {
                write!(formatter, "Poem AOP invocation failed: {source}")
            }
            Self::RequestAlreadyTaken => {
                formatter.write_str("Poem AOP target consumed the request more than once")
            }
            Self::ResponseAlreadyTaken => {
                formatter.write_str("Poem AOP response envelope was already consumed")
            }
        }
    }
}

impl Error for PoemAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}

impl ResponseError for PoemAopError {
    fn status(&self) -> StatusCode {
        PoemAopError::status(self)
    }

    fn as_response(&self) -> Response {
        Response::builder()
            .status(self.status())
            .body(Body::from_string(self.safe_message().to_owned()))
    }
}
