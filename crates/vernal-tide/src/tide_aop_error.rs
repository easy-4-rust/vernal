//! Tide 严格 Send-AOP 错误映射对象。

use std::{error::Error, fmt};

use tide::{Response, StatusCode};
use vernal_aop::InvocationError;
use vernal_web::WebFailure;

/// 将 Tide 严格 AOP 失败转换为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum TideAopError {
    /// 显式低基数路径模式为空。
    MissingRouteMetadata,
    /// `http-types` 到 `http` 1.x 的元数据转换失败。
    RequestSnapshot {
        /// 发生转换失败的稳定字段名，不包含用户输入。
        field: &'static str,
    },
    /// AOP 调用链、取消、截止时间或目标调用失败。
    Invocation {
        /// 原始结构化调用错误。
        source: InvocationError,
    },
    /// 目标被重复推进，owned Request 已经被消费。
    RequestAlreadyTaken,
    /// 目标被重复推进，借用的 Tide `Next` 已经被消费。
    NextAlreadyTaken,
    /// 目标错误地向同一响应信封写入两次。
    ResponseAlreadyStored,
    /// AOP 返回值不是 Tide 响应信封。
    ResponseTypeMismatch,
    /// AOP 返回的响应信封为空或已经被消费。
    ResponseUnavailable,
}

impl TideAopError {
    /// 创建不携带原始用户数据的快照转换错误。
    #[must_use]
    pub const fn snapshot_conversion(field: &'static str) -> Self {
        Self::RequestSnapshot { field }
    }

    /// 包装 AOP 调用错误。
    #[must_use]
    pub const fn invocation(source: InvocationError) -> Self {
        Self::Invocation { source }
    }

    /// 返回安全 HTTP 状态码。
    #[must_use]
    pub fn status(&self) -> StatusCode {
        if let Some(failure) = self.web_failure() {
            return StatusCode::try_from(failure.problem().status())
                .unwrap_or(StatusCode::InternalServerError);
        }
        match self {
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => StatusCode::RequestTimeout,
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => StatusCode::GatewayTimeout,
            _ => StatusCode::InternalServerError,
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
            Self::MissingRouteMetadata => "Tide route pattern is unavailable",
            Self::RequestSnapshot { .. } => "Tide request metadata conversion failed",
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal AOP invocation failed",
            Self::RequestAlreadyTaken => "Tide request was already consumed",
            Self::NextAlreadyTaken => "Tide middleware chain was already consumed",
            Self::ResponseAlreadyStored => "Tide response was produced more than once",
            Self::ResponseTypeMismatch => "Tide AOP returned an incompatible response",
            Self::ResponseUnavailable => "Tide response is unavailable",
        }
    }

    /// 构造稳定 Tide 原生错误响应。
    #[must_use]
    pub fn response(&self) -> Response {
        let mut response = Response::new(self.status());
        response.set_body(self.safe_message().to_owned());
        response
    }

    /// 从目标错误中恢复框架中立 Web 失败。
    fn web_failure(&self) -> Option<&WebFailure> {
        match self {
            Self::Invocation {
                source: InvocationError::Target { source },
            } => source.downcast_ref::<WebFailure>(),
            _ => None,
        }
    }
}

impl fmt::Display for TideAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRouteMetadata => {
                formatter.write_str("Tide strict AOP has no route pattern")
            }
            Self::RequestSnapshot { field } => {
                write!(formatter, "Tide request {field} cannot be converted")
            }
            Self::Invocation { source } => {
                write!(formatter, "Tide AOP invocation failed: {source}")
            }
            Self::RequestAlreadyTaken => formatter.write_str("Tide AOP consumed Request twice"),
            Self::NextAlreadyTaken => formatter.write_str("Tide AOP consumed Next twice"),
            Self::ResponseAlreadyStored => formatter.write_str("Tide AOP produced two responses"),
            Self::ResponseTypeMismatch => formatter.write_str("Tide AOP returned a foreign value"),
            Self::ResponseUnavailable => formatter.write_str("Tide AOP response is unavailable"),
        }
    }
}

impl Error for TideAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}
