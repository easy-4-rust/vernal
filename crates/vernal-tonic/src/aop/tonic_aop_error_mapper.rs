//! Tonic AOP 错误映射对象。

use std::error::Error;

use tonic::Status;
use vernal_aop::InvocationError;
use vernal_tower::AopServiceError;
use vernal_web::WebFailure;

use crate::TonicStatusMapper;

/// 把 Vernal AOP 失败映射为原生 gRPC `Status`。
pub struct TonicAopErrorMapper;

impl TonicAopErrorMapper {
    /// 生成不会泄漏 Token、存储错误或组件拓扑的稳定状态。
    #[must_use]
    pub fn from_service_error<E>(error: &AopServiceError<E>) -> Status
    where
        E: Error + 'static,
    {
        if let Some(failure) = Self::web_failure(error) {
            return TonicStatusMapper::from_problem(failure.problem());
        }
        match error {
            AopServiceError::Invocation(InvocationError::Cancelled) => {
                Status::cancelled("Request was cancelled")
            }
            AopServiceError::Invocation(InvocationError::DeadlineExceeded) => {
                Status::deadline_exceeded("Request deadline exceeded")
            }
            AopServiceError::MissingApplicationContext => {
                Status::internal("Vernal application context is unavailable")
            }
            AopServiceError::MissingRequestScope => {
                Status::internal("Vernal request scope is unavailable")
            }
            AopServiceError::MissingRouteMetadata => {
                Status::internal("Tonic route metadata is unavailable")
            }
            AopServiceError::Upstream(_) => Status::internal("Tonic upstream service failed"),
            _ => Status::internal("Vernal AOP invocation failed"),
        }
    }

    /// 从调用目标错误中恢复框架中立 Web 失败。
    fn web_failure<E>(error: &AopServiceError<E>) -> Option<&WebFailure> {
        match error {
            AopServiceError::Invocation(InvocationError::Target { source }) => {
                source.downcast_ref::<WebFailure>()
            }
            _ => None,
        }
    }
}
