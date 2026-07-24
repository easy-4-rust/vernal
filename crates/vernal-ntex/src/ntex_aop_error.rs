//! Ntex Local-AOP 错误映射对象。

use std::{error::Error, fmt};

use ntex::{
    http::StatusCode,
    web::{HttpResponse, WebResponse},
};
use vernal_aop::LocalInvocationError;
use vernal_web::WebFailure;

/// 将 Ntex 严格 Local-AOP 失败转换为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum NtexAopError {
    /// 严格中间件没有声明低基数 Resource Pattern。
    MissingResourcePattern,
    /// Local-AOP 调用链、取消、截止时间或目标执行失败。
    Invocation {
        /// 原始结构化本地调用错误。
        source: LocalInvocationError,
    },
    /// Local-AOP 目标被重复推进，原生请求已经被消费。
    RequestAlreadyTaken,
    /// Ntex 目标错误地写入了第二个响应。
    ResponseAlreadyStored,
    /// Local-AOP 成功结束，但目标没有生成原生响应。
    ResponseUnavailable,
    /// 拦截器返回了不属于当前调用的响应信封。
    ResponseEnvelopeMismatch,
}

impl NtexAopError {
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
            Self::MissingResourcePattern => "Ntex resource pattern is unavailable",
            Self::Invocation {
                source: LocalInvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: LocalInvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal Local-AOP invocation failed",
            Self::RequestAlreadyTaken => "Ntex request was already consumed",
            Self::ResponseAlreadyStored => "Ntex response was produced more than once",
            Self::ResponseUnavailable => "Ntex response is unavailable",
            Self::ResponseEnvelopeMismatch => "Ntex response envelope does not match the request",
        }
    }

    /// 构造不暴露内部 source 链的 Ntex HTTP 响应。
    #[must_use]
    pub fn response(&self) -> HttpResponse {
        HttpResponse::build(self.status()).body(self.safe_message().to_owned())
    }

    /// 用 AOP 失败替换已经存在的原生响应，同时保留原请求身份。
    #[must_use]
    pub fn replace_response(&self, response: WebResponse) -> WebResponse {
        response.into_response(self.response())
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

impl fmt::Display for NtexAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResourcePattern => {
                formatter.write_str("Ntex strict AOP has no resource pattern")
            }
            Self::Invocation { source } => {
                write!(formatter, "Ntex Local-AOP invocation failed: {source}")
            }
            Self::RequestAlreadyTaken => {
                formatter.write_str("Ntex Local-AOP target consumed the request more than once")
            }
            Self::ResponseAlreadyStored => {
                formatter.write_str("Ntex Local-AOP target produced more than one response")
            }
            Self::ResponseUnavailable => {
                formatter.write_str("Ntex Local-AOP target produced no response")
            }
            Self::ResponseEnvelopeMismatch => {
                formatter.write_str("Ntex Local-AOP returned a foreign response envelope")
            }
        }
    }
}

impl Error for NtexAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}
