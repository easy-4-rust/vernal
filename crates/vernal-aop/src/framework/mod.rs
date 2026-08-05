//! 框架模块。
//!
//! 对应 spring-aop `framework` 包。
//! 提供 AOP 代理和框架核心功能。

pub mod adapter;
pub mod advised;
pub mod advised_support;
pub mod advised_support_listener;
pub mod advisor_chain_factory;
pub mod aop_config_exception;
pub mod aop_context;
pub mod aop_infrastructure_bean;
pub mod aop_proxy;
pub mod aop_proxy_utils;
pub mod autoproxy;
pub mod default_aop_proxy_factory;
pub mod interceptor_and_dynamic_method_matcher;
pub mod proxy_config;
pub mod proxy_creator_support;
pub mod proxy_factory;
pub mod reflective_method_invocation;

// Re-export
pub use adapter::{AdvisorAdapter, AdvisorAdapterRegistry, DefaultAdvisorAdapterRegistry};
pub use advised::Advised;
pub use advised_support::AdvisedSupport;
pub use advised_support_listener::{AdvisedSupportListener, FnAdvisedSupportListener};
pub use advisor_chain_factory::{AdvisorChainFactory, DefaultAdvisorChainFactory};
pub use aop_config_exception::AopConfigException;
pub use aop_context::AopContext;
pub use aop_infrastructure_bean::{AopInfrastructureBean, DefaultAopInfrastructureBean};
pub use aop_proxy::{AopProxy, AopProxyError, AopProxyFactory, FnAopProxy};
pub use aop_proxy_utils::AopProxyUtils;
pub use default_aop_proxy_factory::DefaultAopProxyFactory;
pub use interceptor_and_dynamic_method_matcher::InterceptorAndDynamicMethodMatcher;
pub use proxy_config::ProxyConfig;
pub use proxy_creator_support::ProxyCreatorSupport;
pub use proxy_factory::ProxyFactory;
pub use reflective_method_invocation::ReflectiveMethodInvocation;
