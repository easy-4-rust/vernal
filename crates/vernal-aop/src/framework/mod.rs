//! 框架模块。
//!
//! 对应 spring-aop `framework` 包。
//! 提供 AOP 代理和框架核心功能。

pub mod adapter;
pub mod aop_proxy;
pub mod advisor_chain_factory;
pub mod aspect_instance_factory;
pub mod constructor_interceptor;
pub mod joinpoint;
pub mod proxy_method_invocation;
pub mod raw_target_access;
pub mod target_source_creator;

// Re-export
pub use adapter::{AdvisorAdapter, AdvisorAdapterRegistry, DefaultAdvisorAdapterRegistry};
pub use aop_proxy::{AopProxy, AopProxyError, AopProxyFactory, FnAopProxy};
pub use advisor_chain_factory::{Advised, AdvisorChainFactory, DefaultAdvisorChainFactory};
pub use aspect_instance_factory::{
    AspectInstanceError, AspectInstanceFactory, LazyAspectInstanceFactory,
    MetadataAwareAspectInstanceFactory, SingletonAspectInstanceFactory,
};
pub use constructor_interceptor::{
    ConstructorInterceptor, ConstructorInvocation, ConstructorInvocationError,
    FnConstructorInterceptor,
};
pub use joinpoint::{InvocationChain, Joinpoint, MethodInvocation};
pub use proxy_method_invocation::{
    IntroductionAwareMethodMatcher, AspectJPrecedenceInformation,
    ProxyMethodInvocation, SimpleProxyMethodInvocation,
};
pub use raw_target_access::{
    AdvisedSupportListener, AopInfrastructureBean, AsyncUncaughtExceptionHandler,
    InstantiationModelAwarePointcutAdvisor, MetadataAwarePointcutAdvisor, PoolingConfig,
    RawTargetAccess, Refreshable, ScopedObject, SpringProxy, ThreadLocalTargetSourceStats,
};
pub use target_source_creator::{FnTargetSourceCreator, TargetSourceCreator};
