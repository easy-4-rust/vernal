//! Rocket 严格 AOP 错误映射对象。

use std::{error::Error, fmt};

use rocket::{Request, http::Status, response::status, route::Outcome};
use vernal_aop::InvocationError;
use vernal_web::WebFailure;

/// 将 Rocket Route 严格 AOP 失败映射为稳定、脱敏的原生 Outcome。
#[derive(Debug)]
pub enum RocketAopError {
    /// Rocket Managed State 中没有应用上下文。
    MissingContext,
    /// 请求 Fairing 尚未建立请求 Scope。
    MissingRequestScope,
    /// Handler 执行时 Rocket 没有暴露已匹配 Route。
    MissingRouteMetadata,
    /// Rocket HTTP 模型无法转换为 owned 标准快照。
    RequestSnapshot {
        /// 失败字段的稳定名称，不携带用户输入。
        field: &'static str,
    },
    /// AOP 调用链、取消、截止时间或目标调用失败。
    Invocation {
        /// 原始结构化调用错误。
        source: InvocationError,
    },
    /// Handler 被重复推进，一次性 Data 已被消费。
    DataAlreadyTaken,
    /// AOP 返回值不是 Rocket Outcome 标记。
    OutcomeTypeMismatch,
    /// Handler 未产生 Outcome 或 Outcome 已被消费。
    OutcomeUnavailable,
}

impl RocketAopError {
    /// 创建请求快照转换错误。
    #[must_use]
    pub const fn snapshot_conversion(field: &'static str) -> Self {
        Self::RequestSnapshot { field }
    }

    /// 包装 AOP 调用错误。
    #[must_use]
    pub const fn invocation(source: InvocationError) -> Self {
        Self::Invocation { source }
    }

    /// 返回 Rocket 原生状态码。
    #[must_use]
    pub fn status(&self) -> Status {
        if let Some(failure) = self.web_failure() {
            return Status::new(failure.problem().status());
        }
        match self {
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => Status::RequestTimeout,
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => Status::GatewayTimeout,
            _ => Status::InternalServerError,
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
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::MissingRouteMetadata => "Rocket matched route metadata is unavailable",
            Self::RequestSnapshot { .. } => "Rocket request metadata conversion failed",
            Self::Invocation {
                source: InvocationError::Cancelled,
            } => "Request was cancelled",
            Self::Invocation {
                source: InvocationError::DeadlineExceeded,
            } => "Request deadline exceeded",
            Self::Invocation { .. } => "Vernal AOP invocation failed",
            Self::DataAlreadyTaken => "Rocket request Data was already consumed",
            Self::OutcomeTypeMismatch => "Rocket AOP returned an incompatible outcome",
            Self::OutcomeUnavailable => "Rocket handler outcome is unavailable",
        }
    }

    /// 转换成带稳定文本的 Rocket Success Outcome。
    pub fn outcome<'r>(&self, request: &'r Request<'_>) -> Outcome<'r> {
        Outcome::from(
            request,
            status::Custom(self.status(), self.safe_message().to_owned()),
        )
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

impl fmt::Display for RocketAopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Rocket has no Vernal context"),
            Self::MissingRequestScope => formatter.write_str("Rocket has no Vernal request scope"),
            Self::MissingRouteMetadata => {
                formatter.write_str("Rocket has no matched route metadata")
            }
            Self::RequestSnapshot { field } => {
                write!(formatter, "Rocket request {field} cannot be converted")
            }
            Self::Invocation { source } => {
                write!(formatter, "Rocket AOP invocation failed: {source}")
            }
            Self::DataAlreadyTaken => formatter.write_str("Rocket AOP consumed Data twice"),
            Self::OutcomeTypeMismatch => formatter.write_str("Rocket AOP returned a foreign value"),
            Self::OutcomeUnavailable => formatter.write_str("Rocket Handler returned no Outcome"),
        }
    }
}

impl Error for RocketAopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation { source } => Some(source),
            _ => None,
        }
    }
}
