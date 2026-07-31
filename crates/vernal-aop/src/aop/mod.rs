//! AOP 核心接口模块。
//!
//! 对应 aopalliance 和 spring-aop 根包。

pub mod advice;
pub mod after_advice;
pub mod before_advice;
pub mod dynamic_introduction_advice;
pub mod introduction_advice;
pub mod introduction_interceptor;

// Re-export
pub use advice::{Advice, AspectException};
pub use after_advice::{AfterAdvice, AfterReturningAdvice, ThrowsAdvice};
pub use before_advice::{BeforeAdvice, MethodBeforeAdvice};
pub use dynamic_introduction_advice::DynamicIntroductionAdvice;
pub use introduction_advice::IntroductionAdvice;
pub use introduction_interceptor::IntroductionInterceptor;
