//! 对应 vernal-aop 扩展（无 aspect-rs 对偶）
//! 语义参照 spring-aop：CGLIB/JDK 代理生成 → Interceptor 链适配
//!
//! 切面到拦截器的桥接适配器。
//! 把 `Aspect`（四段式用户门面）自动适配为 `Interceptor`（底层执行原语）。

use std::sync::Arc;

use crate::{Aspect, Interceptor, Invocation, InvocationFuture, Next};

/// 把 [`Aspect`] 自动适配为 [`Interceptor`] 的桥接器。
///
/// 用户实现 `Aspect`（before/after/after_error/around 四段式），
/// `AspectAdapter` 自动将其转换为 vernal-aop 内核的 `Interceptor`。
pub struct AspectAdapter<A: Aspect> {
    aspect: Arc<A>,
}

impl<A: Aspect> AspectAdapter<A> {
    /// 创建切面适配器。
    #[must_use]
    pub fn new(aspect: A) -> Self {
        Self {
            aspect: Arc::new(aspect),
        }
    }

    /// 使用已共享的 Arc 创建适配器。
    #[must_use]
    pub fn shared(aspect: Arc<A>) -> Self {
        Self { aspect }
    }

    /// 获取底层切面的共享引用。
    #[must_use]
    pub fn aspect(&self) -> &A {
        &self.aspect
    }
}

impl<A: Aspect> Interceptor for AspectAdapter<A> {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        // 关键：&self.aspect 的生命周期与 &'a self 绑定，因此 &self.aspect: &'a A
        // 满足 around(&'a self, ..., Next<'a>) 的签名约束。
        self.aspect.around(invocation, next)
    }
}

impl<A: Aspect> Clone for AspectAdapter<A> {
    fn clone(&self) -> Self {
        Self {
            aspect: Arc::clone(&self.aspect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InvocationError, InvocationValue};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingAdapterAspect {
        before_count: std::sync::Arc<AtomicUsize>,
        after_count: std::sync::Arc<AtomicUsize>,
    }

    impl Aspect for CountingAdapterAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            self.before_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }

        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            self.after_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    #[test]
    fn aspect_adapter_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<AspectAdapter<CountingAdapterAspect>>();
        assert_sync::<AspectAdapter<CountingAdapterAspect>>();
    }

    #[test]
    fn aspect_adapter_is_clone() {
        let before = std::sync::Arc::new(AtomicUsize::new(0));
        let after = std::sync::Arc::new(AtomicUsize::new(0));
        let adapter = AspectAdapter::new(CountingAdapterAspect {
            before_count: std::sync::Arc::clone(&before),
            after_count: std::sync::Arc::clone(&after),
        });
        let _cloned = adapter.clone();
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    struct TestAspect;
    impl Aspect for TestAspect {}

    #[test]
    fn aspect_adapter_new() {
        let adapter = AspectAdapter::new(TestAspect);
        let _ = adapter.aspect();
    }

    #[test]
    fn aspect_adapter_shared() {
        let aspect = Arc::new(TestAspect);
        let adapter = AspectAdapter::shared(aspect);
        let _ = adapter.aspect();
    }

    #[tokio::test]
    async fn aspect_adapter_intercept() {
        let adapter = AspectAdapter::new(TestAspect);
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
            }),
        );
        let result = adapter.intercept(inv, next).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod aspect_adapter_tests {
    use super::*;
    use crate::Operation;

    struct TestAspect;
    impl Aspect for TestAspect {}

    #[test]
    fn aspect_adapter_new() {
        let adapter = AspectAdapter::new(TestAspect);
        let _ = adapter.aspect();
    }

    #[test]
    fn aspect_adapter_shared() {
        let aspect = Arc::new(TestAspect);
        let adapter = AspectAdapter::shared(aspect);
        let _ = adapter.aspect();
    }

    #[test]
    fn aspect_adapter_clone() {
        let adapter = AspectAdapter::new(TestAspect);
        let cloned = adapter.clone();
        let _ = cloned.aspect();
    }

    #[tokio::test]
    async fn aspect_adapter_intercept() {
        let adapter = AspectAdapter::new(TestAspect);
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
            }),
        );
        let result = adapter.intercept(inv, next).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod aspect_adapter_final_tests {
    use super::*;
    use crate::Operation;

    struct TestAspect;
    impl Aspect for TestAspect {}

    #[test]
    fn aspect_adapter_aspect() {
        let adapter = AspectAdapter::new(TestAspect);
        let _ = adapter.aspect();
    }

    #[test]
    fn aspect_adapter_clone_and_aspect() {
        let adapter = AspectAdapter::new(TestAspect);
        let cloned = adapter.clone();
        let _ = cloned.aspect();
    }

    #[tokio::test]
    async fn aspect_adapter_intercept_success() {
        let adapter = AspectAdapter::new(TestAspect);
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
            }),
        );
        let result = adapter.intercept(inv, next).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn aspect_adapter_intercept_failure() {
        let adapter = AspectAdapter::new(TestAspect);
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Err(crate::InvocationError::Cancelled) })
            }),
        );
        let result = adapter.intercept(inv, next).await;
        assert!(result.is_err());
    }

    #[test]
    fn aspect_adapter_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AspectAdapter<TestAspect>>();
        assert_sync::<AspectAdapter<TestAspect>>();
    }
}
