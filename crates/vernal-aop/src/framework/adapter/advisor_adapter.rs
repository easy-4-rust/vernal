//! 顾问适配器。
//!
//! 对应 spring-aop `AdvisorAdapter`。
//! 允许扩展 Spring AOP 框架以处理新的顾问和通知类型。

use std::sync::Arc;

use crate::{Advisor, Interceptor};

/// 顾问适配器接口。
///
/// 对应 spring-aop `AdvisorAdapter`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AdvisorAdapter;
///
/// let adapter = MyAdvisorAdapter::new();
/// if adapter.supports_advice(&advice) {
///     let interceptor = adapter.get_interceptor(&advisor);
/// }
/// ```
pub trait AdvisorAdapter: Send + Sync + 'static {
    /// 是否支持给定的通知类型。
    fn supports_advice(&self, advisor: &Advisor) -> bool;

    /// 获取拦截器。
    fn get_interceptor(&self, advisor: &Advisor) -> Arc<dyn Interceptor>;
}

/// 顾问适配器注册表。
///
/// 对应 spring-aop `AdvisorAdapterRegistry`。
pub trait AdvisorAdapterRegistry: Send + Sync + 'static {
    /// 注册顾问适配器。
    fn register_adapter(&mut self, adapter: Box<dyn AdvisorAdapter>);

    /// 获取拦截器列表。
    fn get_interceptors(&self, advisor: &Advisor) -> Vec<Arc<dyn Interceptor>>;
}

/// 默认顾问适配器注册表。
pub struct DefaultAdvisorAdapterRegistry {
    adapters: Vec<Box<dyn AdvisorAdapter>>,
}

impl DefaultAdvisorAdapterRegistry {
    /// 创建新的默认顾问适配器注册表。
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }
}

impl Default for DefaultAdvisorAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvisorAdapterRegistry for DefaultAdvisorAdapterRegistry {
    fn register_adapter(&mut self, adapter: Box<dyn AdvisorAdapter>) {
        self.adapters.push(adapter);
    }

    fn get_interceptors(&self, advisor: &Advisor) -> Vec<Arc<dyn Interceptor>> {
        let mut interceptors = Vec::new();

        for adapter in &self.adapters {
            if adapter.supports_advice(advisor) {
                interceptors.push(adapter.get_interceptor(advisor));
            }
        }

        interceptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestAdapter;

    impl AdvisorAdapter for TestAdapter {
        fn supports_advice(&self, _advisor: &Advisor) -> bool {
            true
        }

        fn get_interceptor(&self, _advisor: &Advisor) -> Arc<dyn Interceptor> {
            // 返回一个简单的拦截器
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
            Arc::new(TestInterceptor)
        }
    }

    #[test]
    fn default_advisor_adapter_registry() {
        let mut registry = DefaultAdvisorAdapterRegistry::new();
        registry.register_adapter(Box::new(TestAdapter));

        let advisor = Advisor::new(AnyPointcut::new(), TestInterceptor, 0);
        let interceptors = registry.get_interceptors(&advisor);
        assert_eq!(interceptors.len(), 1);
    }

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
}
