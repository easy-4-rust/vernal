//! 异步执行切面（对标 Spring 的 @Async）。
//!
//! 通过 AOP 拦截器实现声明式异步执行。
//! 与 `vernal-async` 的 `AsyncTaskExecutor` 配合使用。

use std::sync::Arc;

use vernal_aop::{
    Interceptor, Invocation, InvocationError, InvocationFuture, Next,
};

/// 异步执行配置。
#[derive(Debug, Clone)]
pub struct AsyncConfig {
    /// 执行器名称（空 = 使用默认执行器）
    pub executor_name: String,
    /// 超时时间（秒），0 表示不限制
    pub timeout_secs: u64,
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            executor_name: String::new(),
            timeout_secs: 0,
        }
    }
}

/// 异步执行切面（对标 Spring 的 AsyncAnnotationAdvisor）。
///
/// 通过 AOP 拦截器实现声明式异步执行。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_aspects::{AsyncAspect, AsyncConfig};
///
/// let config = AsyncConfig {
///     executor_name: "custom-executor".to_string(),
///     timeout_secs: 30,
/// };
///
/// let aspect = AsyncAspect::new(config);
/// ```
pub struct AsyncAspect {
    config: AsyncConfig,
}

impl AsyncAspect {
    /// 创建异步执行切面。
    #[must_use]
    pub fn new(config: AsyncConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建异步执行切面。
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(AsyncConfig::default())
    }

    /// 获取异步配置。
    #[must_use]
    pub fn config(&self) -> &AsyncConfig {
        &self.config
    }
}

impl Interceptor for AsyncAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // TODO: 集成 vernal-async 的 AsyncTaskExecutor
            // 当前实现：直接执行目标方法，不做异步化
            // 完整实现需要：
            // 1. 从 invocation context 获取 AsyncTaskExecutor
            // 2. 将目标方法提交到执行器
            // 3. 返回 Future（而非等待结果）

            next.run(invocation).await
        })
    }
}
