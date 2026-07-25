//! 可变与泛型方法织入合同测试组件对象。

use std::sync::Arc;

use vernal_aop::{CancellationToken, InvocationError, InvocationPlanCatalog};

/// 验证独占借用和多次泛型单态化仍使用同一份预编译计划的状态型组件。
#[derive(vernal_macros::Component)]
#[component(scope = "transient", aop)]
pub(crate) struct StatefulService {
    invocation_plans: Arc<InvocationPlanCatalog>,
    cancellation: Arc<CancellationToken>,
    #[component(default)]
    value: i32,
}

impl StatefulService {
    /// 在完整 Around 链内部独占修改瞬时组件状态。
    #[vernal_macros::intercept(component = "StatefulService", tags = ["mutable"])]
    pub(crate) async fn replace(&mut self, value: i32) -> Result<i32, InvocationError> {
        tokio::task::yield_now().await;
        let previous = self.value;
        self.value = value;
        Ok(previous)
    }

    /// 用同一个 Operation 计划处理多个满足擦除边界的泛型返回类型。
    #[vernal_macros::intercept(component = "StatefulService", tags = ["generic"])]
    pub(crate) async fn echo<T>(&self, value: T) -> Result<T, InvocationError>
    where
        T: Send + Sync + 'static,
    {
        tokio::task::yield_now().await;
        Ok(value)
    }

    /// 验证显式生命周期泛型不会逃出当前借用调用。
    #[vernal_macros::intercept(component = "StatefulService", tags = ["generic", "lifetime"])]
    pub(crate) async fn clone_borrowed<'a, T>(&self, value: &'a T) -> Result<T, InvocationError>
    where
        T: Clone + Send + Sync + 'static,
    {
        tokio::task::yield_now().await;
        Ok(value.clone())
    }

    /// 验证 const 泛型参与返回类型恢复而不改变 Operation 身份。
    #[vernal_macros::intercept(component = "StatefulService", tags = ["generic", "const"])]
    pub(crate) async fn echo_array<const N: usize>(
        &self,
        value: [u8; N],
    ) -> Result<[u8; N], InvocationError> {
        tokio::task::yield_now().await;
        Ok(value)
    }

    /// 返回当前可变状态，用于验证短生命周期独占调用的副作用。
    pub(crate) const fn value(&self) -> i32 {
        self.value
    }
}
