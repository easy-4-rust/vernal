//! 不可变本地调用计划对象。

use std::{rc::Rc, sync::Arc};

use crate::{
    Invocation, LocalInterceptor, LocalInvocationError, LocalInvocationResult,
    LocalInvocationTarget, LocalNext, Operation,
};

/// 针对一个操作预编译的 `!Send` 有序拦截器链。
///
/// 计划本身只保存 `Send + Sync` 的不可变拦截器对象，可以跨 Worker 共享；每次
/// [`Self::invoke`] 返回的 Future、目标与擦除值保持线程本地。
#[derive(Clone)]
pub struct LocalInvocationPlan {
    operation: Operation,
    interceptors: Arc<[Arc<dyn LocalInterceptor>]>,
}

impl LocalInvocationPlan {
    /// 由 [`crate::LocalInvocationPlanBuilder`] 创建已排序计划。
    pub(crate) fn new(operation: Operation, interceptors: Vec<Arc<dyn LocalInterceptor>>) -> Self {
        Self {
            operation,
            interceptors: interceptors.into(),
        }
    }

    /// 执行完整本地环绕链，并统一处理 Tokio 取消与截止时间。
    ///
    /// # Errors
    ///
    /// 操作不匹配、调用取消、超过截止时间，或本地拦截器及目标失败时返回
    /// [`LocalInvocationError`]。
    pub async fn invoke(
        &self,
        invocation: Arc<Invocation>,
        target: Rc<LocalInvocationTarget>,
    ) -> LocalInvocationResult {
        if invocation.operation() != &self.operation {
            return Err(LocalInvocationError::PlanMismatch {
                expected: self.operation.clone(),
                actual: invocation.operation().clone(),
            });
        }

        let cancellation = invocation.cancellation().clone();
        let deadline = invocation.deadline();
        let execution = LocalNext::new(&self.interceptors, target.as_ref()).run(invocation);

        if let Some(deadline) = deadline {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => Err(LocalInvocationError::Cancelled),
                () = tokio::time::sleep_until(deadline) => {
                    Err(LocalInvocationError::DeadlineExceeded)
                },
                result = execution => result,
            }
        } else {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => Err(LocalInvocationError::Cancelled),
                result = execution => result,
            }
        }
    }

    /// 返回计划绑定的操作。
    #[must_use]
    pub fn operation(&self) -> &Operation {
        &self.operation
    }

    /// 返回本地拦截器数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// 返回计划是否不包含任何本地拦截器。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }
}
