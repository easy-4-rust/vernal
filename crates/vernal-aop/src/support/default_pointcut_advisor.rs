//! 对应 spring-aop：org.springframework.aop.support.DefaultPointcutAdvisor
//!
//! 默认切点顾问实现（spring-aop 语义，aspect-rs 无对偶）。
//! 最常用的 Advisor 实现，可与任何切点和通知类型配合使用。

use std::sync::Arc;

use crate::{Interceptor, Pointcut, PointcutAdvisor};

/// 默认切点顾问。
///
/// 对应 spring-aop `DefaultPointcutAdvisor`。
/// 可与任何切点和通知类型配合使用，但不适用于引入。
pub struct DefaultPointcutAdvisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn Interceptor>,
    order: i32,
}

impl DefaultPointcutAdvisor {
    /// 创建匹配指定切点的默认顾问。
    pub fn new<I: Interceptor>(pointcut: impl Pointcut, interceptor: I) -> Self {
        Self {
            pointcut: Arc::new(pointcut),
            interceptor: Arc::new(interceptor),
            order: 0,
        }
    }

    /// 使用指定顺序创建顾问。
    pub fn with_order<I: Interceptor>(pointcut: impl Pointcut, interceptor: I, order: i32) -> Self {
        Self {
            pointcut: Arc::new(pointcut),
            interceptor: Arc::new(interceptor),
            order,
        }
    }

    /// 使用已共享的 trait object 创建顾问。
    pub fn shared(
        pointcut: Arc<dyn Pointcut>,
        interceptor: Arc<dyn Interceptor>,
        order: i32,
    ) -> Self {
        Self {
            pointcut,
            interceptor,
            order,
        }
    }
}

impl PointcutAdvisor for DefaultPointcutAdvisor {
    fn pointcut(&self) -> &dyn Pointcut {
        self.pointcut.as_ref()
    }

    fn interceptor(&self) -> &dyn Interceptor {
        self.interceptor.as_ref()
    }

    fn order(&self) -> i32 {
        self.order
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Invocation, InvocationFuture, Next, Operation};
    use std::sync::Arc;

    struct NopInterceptor;

    impl crate::Interceptor for NopInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: Arc<Invocation>,
            next: Next<'a>,
        ) -> InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn default_advisor_matches_all_with_any_pointcut() {
        let advisor = DefaultPointcutAdvisor::new(crate::AnyPointcut::new(), NopInterceptor);
        let op = Operation::new("Svc", "method");
        assert!(advisor.pointcut().matches(&op));
    }

    #[test]
    fn default_advisor_order() {
        let advisor =
            DefaultPointcutAdvisor::with_order(crate::AnyPointcut::new(), NopInterceptor, 42);
        assert_eq!(advisor.order(), 42);
    }

    #[test]
    fn default_advisor_with_component_pointcut() {
        let advisor = DefaultPointcutAdvisor::new(
            crate::ComponentPointcut::new("OrderService"),
            NopInterceptor,
        );

        let op1 = Operation::new("OrderService", "create");
        let op2 = Operation::new("UserService", "create");
        assert!(advisor.pointcut().matches(&op1));
        assert!(!advisor.pointcut().matches(&op2));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::{Invocation, InvocationFuture, Next, Operation};
    use std::sync::Arc;

    struct NopInterceptor;
    impl crate::Interceptor for NopInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: Arc<Invocation>,
            next: Next<'a>,
        ) -> InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn default_advisor_shared() {
        let pointcut = Arc::new(crate::AnyPointcut::new()) as Arc<dyn Pointcut>;
        let interceptor = Arc::new(NopInterceptor) as Arc<dyn Interceptor>;
        let advisor = DefaultPointcutAdvisor::shared(pointcut, interceptor, 10);
        assert_eq!(advisor.order(), 10);
    }

    #[test]
    fn default_advisor_pointcut() {
        let advisor = DefaultPointcutAdvisor::new(crate::AnyPointcut::new(), NopInterceptor);
        let _ = advisor.pointcut();
    }

    #[test]
    fn default_advisor_interceptor() {
        let advisor = DefaultPointcutAdvisor::new(crate::AnyPointcut::new(), NopInterceptor);
        let _ = advisor.interceptor();
    }

    #[test]
    fn default_advisor_with_tag_pointcut() {
        let pc = crate::TagPointcut::new("secured").unwrap();
        let advisor = DefaultPointcutAdvisor::new(pc, NopInterceptor);
        let op = Operation::new("Service", "method").with_metadata(
            crate::OperationMetadata::empty()
                .with_tag("secured")
                .unwrap(),
        );
        assert!(advisor.pointcut().matches(&op));
    }

    #[test]
    fn default_advisor_with_qualifier_pointcut() {
        let pc = crate::QualifierPointcut::new("primary").unwrap();
        let advisor = DefaultPointcutAdvisor::new(pc, NopInterceptor);
        let op = Operation::new("Service", "method").with_metadata(
            crate::OperationMetadata::empty()
                .with_qualifier("primary")
                .unwrap(),
        );
        assert!(advisor.pointcut().matches(&op));
    }
}
