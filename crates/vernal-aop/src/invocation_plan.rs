//! 不可变调用计划对象。

use std::sync::Arc;

use crate::{
    BorrowedInvocationTarget, Interceptor, Invocation, InvocationError, InvocationResult,
    InvocationTarget, Next, Operation, target_ref::TargetRef,
};

/// 针对一个操作预编译的有序拦截器链。
///
/// 计划构建后只保存不可变 `Arc` 切片，可被任意数量的 Tokio task 并发调用，
/// 不需要全局指针映射或运行时修改拦截器列表。
#[derive(Clone)]
pub struct InvocationPlan {
    operation: Operation,
    interceptors: Arc<[Arc<dyn Interceptor>]>,
}

impl InvocationPlan {
    /// 由 [`crate::InvocationPlanBuilder`] 创建已排序计划。
    pub(crate) fn new(operation: Operation, interceptors: Vec<Arc<dyn Interceptor>>) -> Self {
        Self {
            operation,
            interceptors: interceptors.into(),
        }
    }

    /// 执行完整环绕链，并统一处理取消与截止时间。
    ///
    /// 取消和超时包裹整条链，因此即使某个拦截器短路或目标长时间等待，
    /// 调用方仍能获得一致的结构化终止结果。
    ///
    /// # Errors
    ///
    /// 操作不匹配、调用被取消、超过截止时间，或拦截器及目标执行失败时返回
    /// [`InvocationError`]。
    pub async fn invoke(
        &self,
        invocation: Arc<Invocation>,
        target: Arc<InvocationTarget>,
    ) -> InvocationResult {
        self.invoke_target(invocation, TargetRef::Static(target.as_ref()))
            .await
    }

    /// 执行可以独占借用当前异步调用期资源的完整线程安全环绕链。
    ///
    /// 该入口适合 Salvo `FlowCtrl` 一类 Future 满足 `Send`，但必须借用原生
    /// Request、Response 或调用控制器的框架对象。目标借用不会逃出本方法；
    /// 取消、deadline、顺序和错误语义与 [`Self::invoke`] 完全相同。
    ///
    /// # Errors
    ///
    /// 操作不匹配、调用取消、超过截止时间，或拦截器及目标失败时返回
    /// [`InvocationError`]。
    pub async fn invoke_borrowed<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        target: &'a mut (dyn BorrowedInvocationTarget + 'a),
    ) -> InvocationResult {
        self.invoke_target(invocation, TargetRef::Borrowed(target))
            .await
    }

    /// 在同一条内部路径上执行静态目标和借用型目标。
    async fn invoke_target<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        target: TargetRef<'a>,
    ) -> InvocationResult {
        if invocation.operation() != &self.operation {
            return Err(InvocationError::PlanMismatch {
                expected: self.operation.clone(),
                actual: invocation.operation().clone(),
            });
        }

        let cancellation = invocation.cancellation().clone();
        let deadline = invocation.deadline();
        let execution = Next::new(&self.interceptors, target).run(invocation);

        if let Some(deadline) = deadline {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => Err(InvocationError::Cancelled),
                () = tokio::time::sleep_until(deadline) => Err(InvocationError::DeadlineExceeded),
                result = execution => result,
            }
        } else {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => Err(InvocationError::Cancelled),
                result = execution => result,
            }
        }
    }

    /// 返回计划绑定的操作。
    #[must_use]
    pub fn operation(&self) -> &Operation {
        &self.operation
    }

    /// 返回拦截器数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// 返回计划是否不包含任何拦截器。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}
