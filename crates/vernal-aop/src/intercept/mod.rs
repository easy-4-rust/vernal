//! 拦截器模块。
//!
//! 对应 aopalliance `intercept` 包。

pub mod constructor_interceptor;
pub mod invocation;
pub mod joinpoint;

// Re-export
pub use constructor_interceptor::{
    ConstructorInterceptor, ConstructorInvocation, ConstructorInvocationError,
    FnConstructorInterceptor,
};
pub use invocation::Invocation;
pub use joinpoint::{InvocationChain, Joinpoint, MethodInvocation};
