//! 本地切面顾问对象。

use std::sync::Arc;

use crate::{LocalInterceptor, Operation, Pointcut};

/// 将切点、`!Send` 环绕拦截器和顺序组合成本地切面声明。
///
/// 切点和排序规则与 [`crate::Advisor`] 完全一致；只在拦截器调用合同上选择
/// [`LocalInterceptor`]，避免为框架路由重新发明 Operation 匹配语义。
#[derive(Clone)]
pub struct LocalAdvisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn LocalInterceptor>,
    order: i32,
}

impl LocalAdvisor {
    /// 使用具体切点与本地拦截器创建顾问。
    #[must_use]
    pub fn new<P, I>(pointcut: P, interceptor: I, order: i32) -> Self
    where
        P: Pointcut,
        I: LocalInterceptor,
    {
        Self {
            pointcut: Arc::new(pointcut),
            interceptor: Arc::new(interceptor),
            order,
        }
    }

    /// 使用已共享的切点与本地拦截器创建顾问。
    #[must_use]
    pub fn shared(
        pointcut: Arc<dyn Pointcut>,
        interceptor: Arc<dyn LocalInterceptor>,
        order: i32,
    ) -> Self {
        Self {
            pointcut,
            interceptor,
            order,
        }
    }

    /// 返回切点是否匹配操作。
    #[must_use]
    pub fn matches(&self, operation: &Operation) -> bool {
        self.pointcut.matches(operation)
    }

    /// 返回执行顺序；数值越小越先进入、越后退出。
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }

    /// 克隆底层本地拦截器共享指针。
    #[must_use]
    pub fn interceptor(&self) -> Arc<dyn LocalInterceptor> {
        Arc::clone(&self.interceptor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLocalInterceptor;
    impl LocalInterceptor for TestLocalInterceptor {
        fn intercept_local<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: crate::LocalNext<'a>,
        ) -> crate::LocalInvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn local_advisor_new() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        assert_eq!(advisor.order(), 0);
    }

    #[test]
    fn local_advisor_matches() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let op = Operation::new("Service", "method");
        assert!(advisor.matches(&op));
    }

    #[test]
    fn local_advisor_no_match() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Other",
            TestLocalInterceptor,
            0,
        );
        let op = Operation::new("Service", "method");
        assert!(!advisor.matches(&op));
    }

    #[test]
    fn local_advisor_clone() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let cloned = advisor.clone();
        assert_eq!(cloned.order(), 0);
    }

    #[test]
    fn local_advisor_interceptor() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let _interceptor = advisor.interceptor();
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::LocalNext;

    struct TestLocalInterceptor;
    impl LocalInterceptor for TestLocalInterceptor {
        fn intercept_local<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: LocalNext<'a>,
        ) -> crate::LocalInvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn local_advisor_shared() {
        let pointcut = Arc::new(|op: &Operation| op.component() == "Service") as Arc<dyn Pointcut>;
        let interceptor = Arc::new(TestLocalInterceptor);
        let advisor = LocalAdvisor::shared(pointcut, interceptor, 5);
        assert_eq!(advisor.order(), 5);
    }

    #[test]
    fn local_advisor_interceptor_returns_arc() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let interceptor = advisor.interceptor();
        // Verify we got a valid Arc
        let _ = interceptor;
    }
}

#[cfg(test)]
mod local_advisor_tests {
    use super::*;
    use crate::LocalNext;

    struct TestLocalInterceptor;
    impl LocalInterceptor for TestLocalInterceptor {
        fn intercept_local<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: LocalNext<'a>,
        ) -> crate::LocalInvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn local_advisor_new() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            5,
        );
        assert_eq!(advisor.order(), 5);
    }

    #[test]
    fn local_advisor_shared() {
        let pointcut = Arc::new(|op: &Operation| op.component() == "Service") as Arc<dyn Pointcut>;
        let interceptor = Arc::new(TestLocalInterceptor);
        let advisor = LocalAdvisor::shared(pointcut, interceptor, 10);
        assert_eq!(advisor.order(), 10);
    }

    #[test]
    fn local_advisor_matches() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let op = Operation::new("Service", "method");
        assert!(advisor.matches(&op));
    }

    #[test]
    fn local_advisor_no_match() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Other",
            TestLocalInterceptor,
            0,
        );
        let op = Operation::new("Service", "method");
        assert!(!advisor.matches(&op));
    }

    #[test]
    fn local_advisor_clone() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let cloned = advisor.clone();
        assert_eq!(cloned.order(), 0);
    }

    #[test]
    fn local_advisor_interceptor() {
        let advisor = LocalAdvisor::new(
            |op: &Operation| op.component() == "Service",
            TestLocalInterceptor,
            0,
        );
        let _interceptor = advisor.interceptor();
    }
}
