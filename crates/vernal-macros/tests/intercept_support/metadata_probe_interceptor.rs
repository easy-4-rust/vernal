//! 操作元数据观测拦截器对象。

use std::sync::{Arc, Mutex};

use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next, OperationMetadata};

/// 记录真正进入拦截器链的权威 Operation 元数据。
///
/// 测试通过该对象验证宏生成的声明先参与计划编译，再由计划投影到运行时
/// Invocation；它不依赖宏调用点临时构造的身份对象。
#[derive(Clone, Default)]
pub(crate) struct MetadataProbeInterceptor {
    observed: Arc<Mutex<Vec<OperationMetadata>>>,
}

impl MetadataProbeInterceptor {
    /// 创建没有观测记录的拦截器。
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 返回当前观测到的不可变元数据快照。
    pub(crate) fn snapshot(&self) -> Vec<OperationMetadata> {
        self.observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Interceptor for MetadataProbeInterceptor {
    /// 记录声明元数据后继续执行原有调用链。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        self.observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(invocation.operation().metadata().clone());
        next.run(invocation)
    }
}
