//! Gotham 严格 AOP 错误映射对象。

use std::{error::Error, fmt};

use gotham::{
    helpers::http::{Body, response::create_response},
    state::State,
};
use http::{Response, StatusCode};
use vernal_aop::InvocationError;
use vernal_web::WebFailure;

/// 将 Gotham 严格 AOP 失败映射为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum GothamAopError {
    /// 显式低基数路径模式为空。
    MissingRouteMetadata,
    /// AOP 调用链、取消、截止时间或目标调用失败。
    Invocation {
        /// 原始结构化调用错误。
        source: InvocationError,
    },
    /// 最终目标被重复推进，Gotham State 已经被消费。
    StateAlreadyTaken,
    /// 最终目标被重复推进，Gotham Pipeline Chain 已经被消费。
    ChainAlreadyTaken,
    /// 最终目标错误地向响应信封写入两次。
    ResponseAlreadyStored,
    /// AOP 返回值不是 Gotham 响应信封。
    ResponseTypeMismatch,
    /// AOP 返回的响应信封为空或已经被消费。
    ResponseUnavailable,
}

impl GothamAopError {
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
            Self::MissingRouteMetadata => "Gotham route pattern is unavailable",
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal AOP invocation failed",
            Self::StateAlreadyTaken => "Gotham request State was already consumed",
            Self::ChainAlreadyTaken => "Gotham middleware chain was already consumed",
            Self::ResponseAlreadyStored => "Gotham response was produced more than once",
            Self::ResponseTypeMismatch => "Gotham AOP returned an incompatible response",
            Self::ResponseUnavailable => "Gotham response is unavailable",
        }
    }

    /// 构造不暴露内部错误链的 Gotham 原生响应。
    #[must_use]
    pub fn response(&self, state: &State) -> Response<Body> {
        create_response(
            state,
            self.status(),
            gotham::mime::TEXT_PLAIN,
            self.safe_message().to_owned(),
        )
    }

    /// 从目标调用错误中读取框架中立 Web 失败。
    fn web_failure(&self) -> Option<&WebFailure> {
        match self {
            Self::Invocation {
                source: InvocationError::Target { source },
            } => source.downcast_ref::<WebFailure>(),
            _ => None,
        }
    }
}

impl fmt::Display for GothamAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRouteMetadata => {
                formatter.write_str("Gotham strict AOP has no route pattern")
            }
            Self::Invocation { source } => {
                write!(formatter, "Gotham AOP invocation failed: {source}")
            }
            Self::StateAlreadyTaken => formatter.write_str("Gotham AOP consumed State twice"),
            Self::ChainAlreadyTaken => formatter.write_str("Gotham AOP consumed Chain twice"),
            Self::ResponseAlreadyStored => formatter.write_str("Gotham AOP produced two responses"),
            Self::ResponseTypeMismatch => {
                formatter.write_str("Gotham AOP returned a foreign value")
            }
            Self::ResponseUnavailable => formatter.write_str("Gotham AOP response is unavailable"),
        }
    }
}

impl Error for GothamAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}
