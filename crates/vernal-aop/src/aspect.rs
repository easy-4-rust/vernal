//! 对应 aspect-rs：aspect-core/src/aspect.rs::Aspect
//! 语义参照 spring-aop：MethodBeforeAdvice / AfterReturningAdvice / ThrowsAdvice / MethodInterceptor
//!
//! 切面门面 trait（四段式异步通知）。
//! 移植自 aspect-rs 的 Aspect trait，改造为 Tokio-first 异步。
//! `around` 默认实现保留 aspect-rs 的 before→proceed→after 模式。

use std::sync::Arc;

use crate::{Invocation, InvocationError, InvocationFuture, InvocationValue, Next};

/// 切面门面（四段式异步通知）。
///
/// 对应 aspect-rs 的 `Aspect` trait，before/after/after_error/around 四段式
/// 改造为异步。`around` 默认实现保留 aspect-rs 的 before→proceed→after 模式。
///
/// 与现有 [`Interceptor`](crate::Interceptor) 的关系：`Aspect` 是用户友好门面，
/// `Interceptor` 是底层执行原语。通过 [`AspectAdapter`](crate::AspectAdapter) 自动转换。
///
/// # spring-aop 语义映射
///
/// | Aspect 方法 | spring-aop Advice |
/// |---|---|
/// | `before()` | `MethodBeforeAdvice.before()` |
/// | `after()` | `AfterReturningAdvice.afterReturning()` |
/// | `after_error()` | `ThrowsAdvice`（反射）|
/// | `around()` | `MethodInterceptor.invoke()` |
pub trait Aspect: Send + Sync + 'static {
    /// 方法执行前调用（异步）。
    ///
    /// 对应 spring-aop `MethodBeforeAdvice.before()`。
    /// 仅在 `around` 的默认实现中被调用；自定义 `around` 时可忽略。
    fn before<'a>(
        &'a self,
        _inv: &'a Invocation,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>>
    {
        Box::pin(async { Ok(()) })
    }

    /// 方法成功执行后调用（异步）。
    ///
    /// 对应 spring-aop `AfterReturningAdvice.afterReturning()`。
    /// 仅在 `around` 的默认实现中被调用。
    fn after<'a>(
        &'a self,
        _inv: &'a Invocation,
        _result: &'a InvocationValue,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    /// 方法异常时调用（异步）。
    ///
    /// 对应 spring-aop `ThrowsAdvice`（反射调用 afterThrowing 方法）。
    /// 仅在 `around` 的默认实现中被调用。
    fn after_error<'a>(
        &'a self,
        _inv: &'a Invocation,
        _error: &'a InvocationError,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    /// 包裹整个执行的环绕通知（异步）。
    ///
    /// 对应 spring-aop `MethodInterceptor.invoke()`。
    /// 默认实现保留 aspect-rs 的 before→proceed→after/after_error 模式。
    ///
    /// # 默认实现语义
    ///
    /// ```text
    /// before(inv)                    // 前置通知（失败则短路）
    ///   → next.run(inv)              // 执行目标函数
    ///   → Ok: after(inv, result)     // 成功后置
    ///   → Err: after_error(inv, err) // 异常通知
    /// ```
    fn around<'a>(&'a self, inv: Arc<Invocation>, next: Next<'a>) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.before(&inv).await?;
            let result = next.run(inv.clone()).await;
            match &result {
                Ok(value) => self.after(&inv, value).await,
                Err(error) => self.after_error(&inv, error).await,
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingAspect {
        before_count: std::sync::Arc<AtomicUsize>,
        after_count: std::sync::Arc<AtomicUsize>,
        error_count: std::sync::Arc<AtomicUsize>,
    }

    impl Aspect for CountingAspect {
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

        fn after_error<'a>(
            &'a self,
            _inv: &'a Invocation,
            _error: &'a InvocationError,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            self.error_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    #[test]
    fn aspect_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_static<T: 'static>() {}

        assert_send::<CountingAspect>();
        assert_sync::<CountingAspect>();
        assert_static::<CountingAspect>();
    }

    #[test]
    fn aspect_trait_object_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Aspect>>();
        assert_sync::<Box<dyn Aspect>>();
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    struct DefaultAspect;
    impl Aspect for DefaultAspect {}

    #[tokio::test]
    async fn default_before_returns_ok() {
        let aspect = DefaultAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let result = aspect.before(&inv).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn default_after_does_nothing() {
        let aspect = DefaultAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let value: InvocationValue = Box::new(42i32);
        aspect.after(&inv, &value).await;
    }

    #[tokio::test]
    async fn default_after_error_does_nothing() {
        let aspect = DefaultAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let error = InvocationError::Cancelled;
        aspect.after_error(&inv, &error).await;
    }

    struct BeforeFailsAspect;
    impl Aspect for BeforeFailsAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            Box::pin(async { Err(InvocationError::Cancelled) })
        }
    }

    #[tokio::test]
    async fn around_before_fails_shortcuts() {
        let aspect = BeforeFailsAspect;
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as InvocationValue) })
            }),
        );
        let result = aspect.around(inv, next).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod aspect_coverage_tests {
    use super::*;
    use crate::Operation;

    struct CustomAspect;
    impl Aspect for CustomAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            Box::pin(async { Ok(()) })
        }

        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async {})
        }

        fn after_error<'a>(
            &'a self,
            _inv: &'a Invocation,
            _error: &'a InvocationError,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async {})
        }
    }

    #[tokio::test]
    async fn custom_aspect_before() {
        let aspect = CustomAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let result = aspect.before(&inv).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn custom_aspect_after() {
        let aspect = CustomAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let value: InvocationValue = Box::new(42i32);
        aspect.after(&inv, &value).await;
    }

    #[tokio::test]
    async fn custom_aspect_after_error() {
        let aspect = CustomAspect;
        let inv = Invocation::new(Operation::new("Service", "method"));
        let error = InvocationError::Cancelled;
        aspect.after_error(&inv, &error).await;
    }

    #[tokio::test]
    async fn custom_aspect_around_success() {
        let aspect = CustomAspect;
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as InvocationValue) })
            }),
        );
        let result = aspect.around(inv, next).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn custom_aspect_around_failure() {
        let aspect = CustomAspect;
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Err(InvocationError::Cancelled) })
            }),
        );
        let result = aspect.around(inv, next).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod aspect_final_tests {
    use super::*;
    use crate::Operation;

    struct CustomAspect;
    impl Aspect for CustomAspect {
        fn before<'a>(
            &'a self,
            _inv: &'a Invocation,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), InvocationError>> + Send + 'a>,
        > {
            Box::pin(async { Ok(()) })
        }

        fn after<'a>(
            &'a self,
            _inv: &'a Invocation,
            _result: &'a InvocationValue,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async {})
        }

        fn after_error<'a>(
            &'a self,
            _inv: &'a Invocation,
            _error: &'a InvocationError,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async {})
        }
    }

    #[tokio::test]
    async fn custom_aspect_around_success() {
        let aspect = CustomAspect;
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Ok(Box::new(42i32) as InvocationValue) })
            }),
        );
        let result = aspect.around(inv, next).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn custom_aspect_around_failure() {
        let aspect = CustomAspect;
        let inv = Arc::new(Invocation::new(Operation::new("Service", "method")));
        let next = Next::new(
            &[],
            crate::target_ref::TargetRef::Static(&|_| {
                Box::pin(async { Err(InvocationError::Cancelled) })
            }),
        );
        let result = aspect.around(inv, next).await;
        assert!(result.is_err());
    }

    #[test]
    fn aspect_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_static<T: 'static>() {}

        assert_send::<CustomAspect>();
        assert_sync::<CustomAspect>();
        assert_static::<CustomAspect>();
    }

    #[test]
    fn aspect_trait_object_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Aspect>>();
        assert_sync::<Box<dyn Aspect>>();
    }
}
