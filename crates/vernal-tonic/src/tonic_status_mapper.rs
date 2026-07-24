//! Tonic Status 映射对象。

use tonic::{Code, Status};
use vernal_web::{ProblemDetails, ProblemKind};

use crate::TonicRequestError;

/// 将框架中立问题与基础设施错误映射为稳定的 gRPC `Status`。
pub struct TonicStatusMapper;

impl TonicStatusMapper {
    /// 映射框架中立问题详情。
    #[must_use]
    pub fn from_problem(problem: &ProblemDetails) -> Status {
        let code = match problem.kind() {
            ProblemKind::ClientInput => Code::InvalidArgument,
            ProblemKind::Unauthenticated => Code::Unauthenticated,
            ProblemKind::PolicyDenied => Code::PermissionDenied,
            ProblemKind::Application => Code::FailedPrecondition,
            ProblemKind::Infrastructure => Code::Internal,
            ProblemKind::Transport => Code::Unavailable,
        };
        Status::new(code, problem.detail().unwrap_or_else(|| problem.title()))
    }

    /// 将请求能力错误映射为不泄露内部拓扑的状态。
    #[must_use]
    pub fn from_request_error(error: &TonicRequestError) -> Status {
        let message = match error {
            TonicRequestError::MissingContext => "Vernal application context is unavailable",
            TonicRequestError::MissingRequestScope => "Vernal request scope is unavailable",
            TonicRequestError::MissingRequestContext => "Vernal request context is unavailable",
            TonicRequestError::MissingGrpcMethod => "Tonic gRPC method metadata is unavailable",
            TonicRequestError::ComponentResolution { .. } => "Vernal component resolution failed",
        };
        Status::internal(message)
    }
}
