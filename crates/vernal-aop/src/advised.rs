//! 类型安全的被增强对象。

use std::{
    any::{Any, type_name},
    error::Error,
    future::Future,
    sync::Arc,
};

use crate::{
    Invocation, InvocationError, InvocationPlan, InvocationTarget, InvocationValue, Operation,
};

/// 将业务目标与预编译调用计划组合起来。
///
/// `Advised<T>` 不改变 `T` 的内存布局，也不依赖全局实例指针表。宏生成代码、
/// 手工组件和 Web 适配器都可以通过同一类型安全入口执行环绕链。
pub struct Advised<T> {
    target: Arc<T>,
    plan: Arc<InvocationPlan>,
}

impl<T> Advised<T>
where
    T: Send + Sync + 'static,
{
    /// 创建被增强对象。
    #[must_use]
    pub fn new(target: Arc<T>, plan: Arc<InvocationPlan>) -> Self {
        Self { target, plan }
    }

    /// 使用计划绑定的操作创建调用并执行目标函数。
    ///
    /// # Errors
    ///
    /// 拦截器、取消、超时、目标函数失败或返回类型恢复失败时返回
    /// [`InvocationError`]。
    pub async fn invoke<R, E, F, Fut>(&self, function: F) -> Result<R, InvocationError>
    where
        R: Any + Send + Sync,
        E: Error + Send + Sync + 'static,
        F: Fn(Arc<T>, Arc<Invocation>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let invocation = Invocation::new(self.plan.operation().clone()).shared();
        self.invoke_with(invocation, function).await
    }

    /// 使用调用方准备的上下文、取消令牌和截止时间执行目标函数。
    ///
    /// # Errors
    ///
    /// 调用操作与计划不匹配，或执行链失败时返回 [`InvocationError`]。
    pub async fn invoke_with<R, E, F, Fut>(
        &self,
        invocation: Arc<Invocation>,
        function: F,
    ) -> Result<R, InvocationError>
    where
        R: Any + Send + Sync,
        E: Error + Send + Sync + 'static,
        F: Fn(Arc<T>, Arc<Invocation>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let advised_target = Arc::clone(&self.target);
        let target: Arc<InvocationTarget> = Arc::new(move |invocation| {
            let advised_target = Arc::clone(&advised_target);
            let future = function(advised_target, invocation);
            Box::pin(async move {
                future
                    .await
                    .map(|value| Box::new(value) as InvocationValue)
                    .map_err(InvocationError::target)
            })
        });

        let value = self.plan.invoke(invocation, target).await?;
        value
            .downcast::<R>()
            .map(|value| *value)
            .map_err(|_| InvocationError::ReturnTypeMismatch {
                expected: type_name::<R>(),
            })
    }

    /// 返回原始业务目标。
    #[must_use]
    pub fn target(&self) -> &Arc<T> {
        &self.target
    }

    /// 返回不可变调用计划。
    #[must_use]
    pub fn plan(&self) -> &Arc<InvocationPlan> {
        &self.plan
    }

    /// 返回计划绑定的操作。
    #[must_use]
    pub fn operation(&self) -> &Operation {
        self.plan.operation()
    }
}
