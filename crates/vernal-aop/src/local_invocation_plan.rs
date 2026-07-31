//! 不可变本地调用计划对象。

use std::{rc::Rc, sync::Arc};

use crate::{
    BorrowedLocalInvocationTarget, Invocation, LocalInterceptor, LocalInvocationError,
    LocalInvocationResult, LocalInvocationTarget, LocalNext, Operation,
    local_target_ref::LocalTargetRef,
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
        self.invoke_target(invocation, LocalTargetRef::Static(target.as_ref()))
            .await
    }

    /// 执行可以借用当前 Worker 调用期资源的完整本地环绕链。
    ///
    /// 该入口用于 Ntex `ServiceCtx` 一类带非静态生命周期的框架对象。目标借用
    /// 不会逃出返回 Future；取消、deadline、顺序与错误语义和 [`Self::invoke`]
    /// 完全相同。
    ///
    /// # Errors
    ///
    /// 操作不匹配、调用取消、超过截止时间，或本地拦截器及目标失败时返回
    /// [`LocalInvocationError`]。
    pub async fn invoke_borrowed<'a>(
        &self,
        invocation: Arc<Invocation>,
        target: Rc<dyn BorrowedLocalInvocationTarget + 'a>,
    ) -> LocalInvocationResult {
        self.invoke_target(invocation, LocalTargetRef::Borrowed(target.as_ref()))
            .await
    }

    /// 在同一条内部路径上执行静态目标和借用型目标。
    async fn invoke_target<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        target: LocalTargetRef<'a>,
    ) -> LocalInvocationResult {
        if invocation.operation() != &self.operation {
            return Err(LocalInvocationError::PlanMismatch {
                expected: self.operation.clone(),
                actual: invocation.operation().clone(),
            });
        }

        // Local 执行平面与 Send 平面使用同一声明投影语义，避免 Actix/Ntex 目标看到
        // 与其他 Adapter 不同的标签或限定符。
        let invocation = invocation.for_declared_operation(self.operation.clone());
        let cancellation = invocation.cancellation().clone();
        let deadline = invocation.deadline();
        let execution = LocalNext::new(&self.interceptors, target).run(invocation);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn local_invocation_plan_operation() {
        let op = Operation::new("Service", "method");
        let plan = LocalInvocationPlan::new(op.clone(), vec![]);
        assert_eq!(plan.operation().component(), "Service");
        assert_eq!(plan.operation().method(), "method");
    }

    #[test]
    fn local_invocation_plan_clone() {
        let op = Operation::new("Service", "method");
        let plan = LocalInvocationPlan::new(op.clone(), vec![]);
        let cloned = plan.clone();
        assert_eq!(cloned.operation().component(), "Service");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::{Operation, LocalInvocationFuture};

    struct TestLocalInterceptor;
    impl LocalInterceptor for TestLocalInterceptor {
        fn intercept_local<'a>(&'a self, invocation: Arc<Invocation>, next: LocalNext<'a>) -> LocalInvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn local_invocation_plan_len_empty() {
        let op = Operation::new("Service", "method");
        let plan = LocalInvocationPlan::new(op, vec![]);
        assert_eq!(plan.len(), 0);
        assert!(plan.is_empty());
    }

    #[test]
    fn local_invocation_plan_len_non_empty() {
        let op = Operation::new("Service", "method");
        let interceptor: Arc<dyn LocalInterceptor> = Arc::new(TestLocalInterceptor);
        let plan = LocalInvocationPlan::new(op, vec![interceptor]);
        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[tokio::test]
    async fn local_invoke_target_mismatch_returns_error() {
        let op = Operation::new("Service", "method");
        let plan = LocalInvocationPlan::new(op, vec![]);
        let different_op = Operation::new("Other", "method");
        let invocation = Arc::new(Invocation::new(different_op));
        let target: Rc<LocalInvocationTarget> = Rc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::LocalInvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn local_invoke_empty_plan_returns_target_result() {
        let op = Operation::new("Service", "method");
        let plan = LocalInvocationPlan::new(op.clone(), vec![]);
        let invocation = Arc::new(Invocation::new(op));
        let target: Rc<LocalInvocationTarget> = Rc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::LocalInvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }
}
