//! 顾问链工厂。
//!
//! 对应 spring-aop `AdvisorChainFactory`。
//! 创建拦截器链。

use std::sync::Arc;

use crate::{Advisor, Interceptor, Operation};

/// 顾问链工厂接口。
///
/// 对应 spring-aop `AdvisorChainFactory`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AdvisorChainFactory;
///
/// let factory = DefaultAdvisorChainFactory::new();
/// let interceptors = factory.get_interceptors(&advised, &operation);
/// ```
pub trait AdvisorChainFactory: Send + Sync + 'static {
    /// 获取拦截器链。
    fn get_interceptors(
        &self,
        advised: &dyn Advised,
        operation: &Operation,
    ) -> Vec<Arc<dyn Interceptor>>;
}

/// 默认顾问链工厂。
///
/// 对应 spring-aop `DefaultAdvisorChainFactory`。
pub struct DefaultAdvisorChainFactory;

impl DefaultAdvisorChainFactory {
    /// 创建新的默认顾问链工厂。
    pub fn new() -> Self {
        Self
    }
}

impl AdvisorChainFactory for DefaultAdvisorChainFactory {
    fn get_interceptors(
        &self,
        advised: &dyn Advised,
        operation: &Operation,
    ) -> Vec<Arc<dyn Interceptor>> {
        let advisors = advised.get_advisors();
        let mut interceptors = Vec::new();

        for advisor in advisors {
            if advisor.matches(operation) {
                interceptors.push(advisor.interceptor());
            }
        }

        interceptors
    }
}

impl Default for DefaultAdvisorChainFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Advised trait（简化版）。
///
/// 对应 spring-aop `Advised`。
pub trait Advised: Send + Sync + 'static {
    /// 获取顾问列表。
    fn get_advisors(&self) -> Vec<&Advisor>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;

    impl Interceptor for TestInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: std::sync::Arc<crate::Invocation>,
            next: crate::Next<'a>,
        ) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestAdvised {
        advisors: Vec<Advisor>,
    }

    impl TestAdvised {
        fn new(advisors: Vec<Advisor>) -> Self {
            Self { advisors }
        }
    }

    impl Advised for TestAdvised {
        fn get_advisors(&self) -> Vec<&Advisor> {
            self.advisors.iter().collect()
        }
    }

    #[test]
    fn default_advisor_chain_factory() {
        let factory = DefaultAdvisorChainFactory::new();
        let interceptor = TestInterceptor;
        let advisor = Advisor::new(AnyPointcut::new(), interceptor, 0);
        let advised = TestAdvised::new(vec![advisor]);
        let op = Operation::new("Service", "method");

        let interceptors = factory.get_interceptors(&advised, &op);
        assert_eq!(interceptors.len(), 1);
    }
}
