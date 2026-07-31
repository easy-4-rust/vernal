#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 异步切面与拦截内核。"]

mod advisor;
mod and_pointcut;
mod any_pointcut;
mod aop_component;
mod aspect;
mod aspect_adapter;
mod aspect_error;
mod aspect_rs_adapter;
mod class_filter;
mod method_matcher;
mod target_source;
pub mod target_source_error;
mod singleton_target_source;
mod lazy_target_source;
mod borrowed_invocation_argument;
mod borrowed_invocation_future_target;
mod borrowed_invocation_target;
mod borrowed_local_invocation_target;
mod component_pointcut;
mod interceptor;
mod introduction_advisor;
mod introduction_info;
mod invocation_context;
mod invocation_error;
mod invocation_id;
mod invocation_output;
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
mod method_pointcut;
mod next;
mod not_pointcut;
mod operation;
mod operation_declaration_set;
mod operation_metadata;
mod operation_metadata_conflict_error;
mod operation_metadata_error;
mod operation_pointcut;
mod or_pointcut;
mod owned_invocation_argument;
pub mod pointcut;
mod pointcut_advisor;
mod pointcut_ext;
mod qualifier_pointcut;
mod shared_invocation_argument;
mod simple_call_result;
mod simple_interceptor;
mod simple_interceptor_chain;
mod simple_invocation_context;
mod proxy_method_invocation;
mod spring_proxy;
mod tag_pointcut;
mod target_ref;
mod true_class_filter;
mod true_method_matcher;
mod true_pointcut;

pub mod aop;
pub mod aspectj;
pub mod framework;
pub mod intercept;
pub mod support;

pub use framework::Advised;
pub use advisor::Advisor;
pub use and_pointcut::AndPointcut;
pub use any_pointcut::AnyPointcut;
pub use aop_component::AopComponent;
pub use aspect::Aspect;
pub use aspect_adapter::AspectAdapter;
pub use aspect_error::AspectError;
pub use aspect_rs_adapter::AspectRsAdapter;
pub use class_filter::{ClassFilter, ClassFilterFactory, TrueClassFilter, FnClassFilter};
pub use method_matcher::{MethodMatcher, MethodMatcherFactory, TrueMethodMatcher, StaticMethodMatcher, DynamicMethodMatcher};
pub use target_source::{TargetSource, TargetClassAware};
pub use target_source_error::TargetSourceError;
pub use singleton_target_source::SingletonTargetSource;
pub use lazy_target_source::LazyTargetSource;
pub use borrowed_invocation_argument::BorrowedInvocationArgument;
pub use borrowed_invocation_future_target::BorrowedInvocationFutureTarget;
pub use borrowed_invocation_target::BorrowedInvocationTarget;
pub use borrowed_local_invocation_target::BorrowedLocalInvocationTarget;
pub use component_pointcut::ComponentPointcut;
pub use support::DefaultPointcutAdvisor;
pub use interceptor::Interceptor;

// 业务切面直接 re-export（基于 aspect-rs aspect-std）
// 通过 AspectRsAdapter 可将这些同步 Aspect 适配为异步 Interceptor
pub use aspect_std::{
    CircuitState, LoggingAspect, TimingAspect, MetricsAspect, CachingAspect,
    RateLimitAspect, CircuitBreakerAspect, AuthorizationAspect, AllowlistAspect,
    AuthMode, ValidationAspect, ValidationRule,
};
pub use aspect_std::validation::{NotEmptyValidator, RangeValidator, CustomValidator};
pub use introduction_advisor::IntroductionAdvisor;
pub use introduction_info::IntroductionInfo;
pub use intercept::Invocation;
pub use invocation_context::InvocationContext;
pub use invocation_error::InvocationError;
pub use invocation_id::InvocationId;
pub use invocation_output::InvocationOutput;
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
pub use method_pointcut::MethodPointcut;
pub use next::Next;
pub use not_pointcut::NotPointcut;
pub use operation::Operation;
pub use operation_metadata::OperationMetadata;
pub use operation_metadata_conflict_error::OperationMetadataConflictError;
pub use operation_metadata_error::OperationMetadataError;
pub use operation_pointcut::OperationPointcut;
pub use or_pointcut::OrPointcut;
pub use owned_invocation_argument::OwnedInvocationArgument;
pub use pointcut::Pointcut;
pub use pointcut_advisor::PointcutAdvisor;
pub use pointcut_ext::PointcutExt;
pub use qualifier_pointcut::QualifierPointcut;
pub use shared_invocation_argument::SharedInvocationArgument;
pub use simple_call_result::SimpleCallResult;
pub use simple_interceptor::SimpleInterceptor;
pub use simple_interceptor_chain::SimpleInterceptorChain;
pub use simple_invocation_context::SimpleInvocationContext;
pub use tag_pointcut::TagPointcut;
/// AOP 组件和宏共用的 Tokio 协作取消令牌。
pub use tokio_util::sync::CancellationToken;

/// 返回当前 `AOP` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
