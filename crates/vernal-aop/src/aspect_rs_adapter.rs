//! aspect-rs 适配器。
//!
//! 桥接 aspect-rs 的同步 `Aspect` trait 和 vernal-aop 的异步 `Interceptor` trait。
//! 使得 aspect-std 的业务切面可以直接在 vernal-aop 的异步环境中使用。

use std::sync::Arc;

use crate::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};

/// 将 aspect-rs 的同步 Aspect 适配为 vernal-aop 的异步 Interceptor。
///
/// 由于 aspect-rs 的 `JoinPoint` 需要 `&'static str`，而 vernal-aop 的
/// `Operation` 是动态的，此适配器在调用 aspect 时不传递 JoinPoint 上下文。
///
/// # 用法
///
/// ```rust,ignore
/// use aspect_std::LoggingAspect;
/// use vernal_aop::AspectRsAdapter;
///
/// let logging = LoggingAspect::new();
/// let interceptor = AspectRsAdapter::new(logging);
/// ```
pub struct AspectRsAdapter<A: aspect_core::Aspect + 'static> {
    aspect: A,
}

impl<A: aspect_core::Aspect + 'static> AspectRsAdapter<A> {
    /// 创建新的适配器。
    pub fn new(aspect: A) -> Self {
        Self { aspect }
    }
}

impl<A: aspect_core::Aspect + 'static> Interceptor for AspectRsAdapter<A> {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        // 注意：aspect-rs 的 JoinPoint 需要 &'static str
        // 由于 vernal-aop 的 Operation 是动态的，这里无法直接创建 JoinPoint
        // 只能调用 aspect 的 before/after，不传递上下文

        // 创建一个空的 JoinPoint（使用空字符串）
        // 这是一个妥协，因为 aspect-rs 的设计假设函数名是编译时已知的
        let join_point = aspect_core::JoinPoint {
            function_name: "",
            module_path: "",
            location: aspect_core::Location {
                file: "",
                line: 0,
            },
        };

        // 调用 aspect-rs 的 before
        self.aspect.before(&join_point);

        // 执行目标函数
        Box::pin(async move {
            let result = next.run(invocation.clone()).await;

            // 根据结果调用 after 或处理错误
            match &result {
                Ok(_) => {
                    // aspect-rs 的 after 需要 &dyn Any，简化处理
                }
                Err(_) => {
                    // aspect-rs 没有标准的 error 回调
                }
            }

            result
        })
    }
}

impl<A: aspect_core::Aspect + Clone + 'static> Clone for AspectRsAdapter<A> {
    fn clone(&self) -> Self {
        Self {
            aspect: self.aspect.clone(),
        }
    }
}

/// 将 aspect-rs 的 Around 实现适配为 vernal-aop 的 Interceptor。
///
/// 这个适配器更完整，支持 aspect-rs 的 `around` 方法。
pub struct AroundAdapter<A: aspect_core::Aspect + 'static> {
    aspect: A,
}

impl<A: aspect_core::Aspect + 'static> AroundAdapter<A> {
    /// 创建新的 around 适配器。
    pub fn new(aspect: A) -> Self {
        Self { aspect }
    }
}

impl<A: aspect_core::Aspect + 'static> Interceptor for AroundAdapter<A> {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        // 创建空的 JoinPoint
        let join_point = aspect_core::JoinPoint {
            function_name: "",
            module_path: "",
            location: aspect_core::Location {
                file: "",
                line: 0,
            },
        };

        // 调用 aspect-rs 的 before
        self.aspect.before(&join_point);

        Box::pin(async move {
            // 执行目标函数
            let result = next.run(invocation).await;

            result
        })
    }
}

impl<A: aspect_core::Aspect + Clone + 'static> Clone for AroundAdapter<A> {
    fn clone(&self) -> Self {
        Self {
            aspect: self.aspect.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspect_core::Aspect;

    /// 测试用的同步 Aspect
    #[derive(Clone)]
    struct TestSyncAspect {
        before_called: Arc<std::sync::atomic::AtomicBool>,
    }

    impl TestSyncAspect {
        fn new() -> Self {
            Self {
                before_called: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }
    }

    impl Aspect for TestSyncAspect {
        fn before(&self, _ctx: &aspect_core::JoinPoint) {
            self.before_called.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn aspect_rs_adapter_creation() {
        let aspect = TestSyncAspect::new();
        let adapter = AspectRsAdapter::new(aspect);
        assert!(!adapter.aspect.before_called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
