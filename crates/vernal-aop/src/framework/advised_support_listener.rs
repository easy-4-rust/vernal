//! 通知支持监听器。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AdvisedSupportListener`。

use crate::Advisor;

/// 通知支持监听器。
///
/// 对应 spring-aop `AdvisedSupportListener`。
///
/// 当通知被激活或停用时调用的监听器。
pub trait AdvisedSupportListener: Send + Sync + 'static {
    /// 通知激活时调用。
    fn advice_activated(&self, advisor: &Advisor);

    /// 通知停用时调用。
    fn advice_deactivated(&self, advisor: &Advisor);
}

/// 基于闭包的通知支持监听器。
pub struct FnAdvisedSupportListener<
    F1: Fn(&Advisor) + Send + Sync + 'static,
    F2: Fn(&Advisor) + Send + Sync + 'static,
> {
    on_activated: F1,
    on_deactivated: F2,
}

impl<F1: Fn(&Advisor) + Send + Sync + 'static, F2: Fn(&Advisor) + Send + Sync + 'static>
    FnAdvisedSupportListener<F1, F2>
{
    /// 创建基于闭包的通知支持监听器。
    pub fn new(on_activated: F1, on_deactivated: F2) -> Self {
        Self {
            on_activated,
            on_deactivated,
        }
    }
}

impl<F1: Fn(&Advisor) + Send + Sync + 'static, F2: Fn(&Advisor) + Send + Sync + 'static>
    AdvisedSupportListener for FnAdvisedSupportListener<F1, F2>
{
    fn advice_activated(&self, advisor: &Advisor) {
        (self.on_activated)(advisor);
    }

    fn advice_deactivated(&self, advisor: &Advisor) {
        (self.on_deactivated)(advisor);
    }
}

impl std::fmt::Debug for dyn AdvisedSupportListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdvisedSupportListener").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn fn_advised_support_listener() {
        struct TestInterceptor;
        impl crate::Interceptor for TestInterceptor {
            fn intercept<'a>(
                &'a self,
                invocation: Arc<crate::Invocation>,
                next: crate::Next<'a>,
            ) -> crate::InvocationFuture<'a> {
                next.run(invocation)
            }
        }

        let activated_count = Arc::new(AtomicUsize::new(0));
        let deactivated_count = Arc::new(AtomicUsize::new(0));

        let activated_count_clone = activated_count.clone();
        let deactivated_count_clone = deactivated_count.clone();

        let listener = FnAdvisedSupportListener::new(
            move |_| {
                activated_count_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_| {
                deactivated_count_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        let advisor =
            crate::Advisor::new(crate::any_pointcut::AnyPointcut::new(), TestInterceptor, 0);

        listener.advice_activated(&advisor);
        assert_eq!(activated_count.load(Ordering::SeqCst), 1);

        listener.advice_deactivated(&advisor);
        assert_eq!(deactivated_count.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn fn_advised_support_listener_new() {
        let activated_count = Arc::new(AtomicUsize::new(0));
        let deactivated_count = Arc::new(AtomicUsize::new(0));

        let activated_count_clone = activated_count.clone();
        let deactivated_count_clone = deactivated_count.clone();

        let _listener = FnAdvisedSupportListener::new(
            move |_| {
                activated_count_clone.fetch_add(1, Ordering::SeqCst);
            },
            move |_| {
                deactivated_count_clone.fetch_add(1, Ordering::SeqCst);
            },
        );
    }
}
