#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 异步切面与拦截内核。"]

mod advised;
mod advisor;
mod aop_component;
mod borrowed_invocation_target;
mod borrowed_local_invocation_target;
mod interceptor;
mod invocation;
mod invocation_context;
mod invocation_error;
mod invocation_id;
mod invocation_plan;
mod invocation_plan_builder;
mod invocation_plan_catalog;
mod invocation_plan_catalog_initialization_error;
mod invocation_result;
mod local_advisor;
mod local_interceptor;
mod local_invocation_error;
mod local_invocation_plan;
mod local_invocation_plan_builder;
mod local_invocation_plan_catalog;
mod local_invocation_result;
mod local_next;
mod local_target_ref;
mod next;
mod operation;
mod pointcut;
mod target_ref;

pub use advised::Advised;
pub use advisor::Advisor;
pub use aop_component::AopComponent;
pub use borrowed_invocation_target::BorrowedInvocationTarget;
pub use borrowed_local_invocation_target::BorrowedLocalInvocationTarget;
pub use interceptor::Interceptor;
pub use invocation::Invocation;
pub use invocation_context::InvocationContext;
pub use invocation_error::InvocationError;
pub use invocation_id::InvocationId;
pub use invocation_plan::InvocationPlan;
pub use invocation_plan_builder::InvocationPlanBuilder;
pub use invocation_plan_catalog::InvocationPlanCatalog;
pub use invocation_plan_catalog_initialization_error::InvocationPlanCatalogInitializationError;
pub use invocation_result::{
    InvocationFuture, InvocationResult, InvocationTarget, InvocationValue,
};
pub use local_advisor::LocalAdvisor;
pub use local_interceptor::LocalInterceptor;
pub use local_invocation_error::LocalInvocationError;
pub use local_invocation_plan::LocalInvocationPlan;
pub use local_invocation_plan_builder::LocalInvocationPlanBuilder;
pub use local_invocation_plan_catalog::LocalInvocationPlanCatalog;
pub use local_invocation_result::{
    LocalInvocationFuture, LocalInvocationResult, LocalInvocationTarget, LocalInvocationValue,
};
pub use local_next::LocalNext;
pub use next::Next;
pub use operation::Operation;
pub use pointcut::Pointcut;
/// AOP 组件和宏共用的 Tokio 协作取消令牌。
pub use tokio_util::sync::CancellationToken;

/// 返回当前 `AOP` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
