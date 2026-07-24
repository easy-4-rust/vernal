//! Salvo 严格 Send-AOP 错误映射对象。

use std::{error::Error, fmt};

use salvo::{Response, http::StatusCode, writing::Text};
use vernal_aop::InvocationError;
use vernal_web::WebFailure;

/// 将 Salvo 严格 AOP 失败转换为稳定、脱敏的原生响应。
#[derive(Debug)]
pub enum SalvoAopError {
    /// 严格 Hoop 没有获得路由匹配后的低基数模板。
    MissingRouteMetadata,
    /// AOP 调用链、取消、截止时间或目标调用失败。
    Invocation {
        /// 原始结构化调用错误，仅供服务端错误链和诊断使用。
        source: InvocationError,
    },
    /// 目标错误地向同一响应信封写入了两次。
    ResponseAlreadyStored,
    /// AOP 成功结束，但返回值不是 Salvo 响应信封。
    ResponseTypeMismatch,
    /// AOP 返回的响应信封已经被消费或从未写入。
    ResponseUnavailable,
}

impl SalvoAopError {
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
            Self::MissingRouteMetadata => "Salvo matched route metadata is unavailable",
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal AOP invocation failed",
            Self::ResponseAlreadyStored => "Salvo response was produced more than once",
            Self::ResponseTypeMismatch => "Salvo AOP returned an incompatible response",
            Self::ResponseUnavailable => "Salvo response is unavailable",
        }
    }

    /// 用稳定状态和脱敏文本覆盖当前 Salvo 响应。
    pub(crate) fn render(&self, response: &mut Response) {
        *response = Response::new();
        response.status_code = Some(self.status());
        response.render(Text::Plain(self.safe_message().to_owned()));
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

impl fmt::Display for SalvoAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRouteMetadata => {
                formatter.write_str("Salvo strict AOP has no matched route metadata")
            }
            Self::Invocation { source } => {
                write!(formatter, "Salvo AOP invocation failed: {source}")
            }
            Self::ResponseAlreadyStored => {
                formatter.write_str("Salvo AOP target produced more than one response")
            }
            Self::ResponseTypeMismatch => {
                formatter.write_str("Salvo AOP returned a non-Salvo response value")
            }
            Self::ResponseUnavailable => {
                formatter.write_str("Salvo AOP returned an empty response envelope")
            }
        }
    }
}

impl Error for SalvoAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}
