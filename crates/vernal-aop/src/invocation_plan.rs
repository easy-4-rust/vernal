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

        // 运行期 Adapter 只需提供稳定身份；进入链前改用计划中经过冲突校验的声明
        // Operation，同时共享原 Invocation 的 ID、Context、取消令牌和 deadline。
        let invocation = invocation.for_declared_operation(self.operation.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    struct TestInterceptor;
    impl Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: std::sync::Arc<Invocation>, next: Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn invocation_plan_operation() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        assert_eq!(plan.operation().component(), "Service");
        assert_eq!(plan.operation().method(), "method");
    }

    #[test]
    fn invocation_plan_clone() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let cloned = plan.clone();
        assert_eq!(cloned.operation().component(), "Service");
    }

    #[test]
    fn invocation_plan_len_empty() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        assert_eq!(plan.len(), 0);
        assert!(plan.is_empty());
    }

    #[test]
    fn invocation_plan_len_non_empty() {
        let op = Operation::new("Service", "method");
        let interceptor: Arc<dyn Interceptor> = Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op, vec![interceptor]);
        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[tokio::test]
    async fn invoke_target_mismatch_returns_error() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        let different_op = Operation::new("Other", "method");
        let invocation = Arc::new(Invocation::new(different_op));
        let target: Arc<InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn invoke_empty_plan_returns_target_result() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let invocation = Arc::new(Invocation::new(op));
        let target: Arc<InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invoke_with_interceptor_calls_chain() {
        let op = Operation::new("Service", "method");
        let interceptor: Arc<dyn Interceptor> = Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op.clone(), vec![interceptor]);
        let invocation = Arc::new(Invocation::new(op));
        let target: Arc<InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod invocation_plan_tests {
    use super::*;
    use crate::Operation;

    struct TestInterceptor;
    impl Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: std::sync::Arc<Invocation>, next: Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn invocation_plan_new() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        assert_eq!(plan.operation().component(), "Service");
    }

    #[test]
    fn invocation_plan_clone() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let cloned = plan.clone();
        assert_eq!(cloned.operation().component(), "Service");
    }

    #[test]
    fn invocation_plan_len() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn invocation_plan_is_empty() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        assert!(plan.is_empty());
    }

    #[test]
    fn invocation_plan_with_interceptor() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op, vec![interceptor]);
        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[tokio::test]
    async fn invocation_plan_invoke() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let invocation = std::sync::Arc::new(Invocation::new(op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invocation_plan_invoke_mismatch() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        let different_op = Operation::new("Other", "method");
        let invocation = std::sync::Arc::new(Invocation::new(different_op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod invocation_plan_final_tests {
    use super::*;
    use crate::Operation;

    struct TestInterceptor;
    impl Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: std::sync::Arc<Invocation>, next: Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[tokio::test]
    async fn invocation_plan_invoke_empty() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let invocation = std::sync::Arc::new(Invocation::new(op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invocation_plan_invoke_with_interceptor() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op.clone(), vec![interceptor]);
        let invocation = std::sync::Arc::new(Invocation::new(op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invocation_plan_invoke_mismatch() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        let different_op = Operation::new("Other", "method");
        let invocation = std::sync::Arc::new(Invocation::new(different_op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_err());
    }

    #[test]
    fn invocation_plan_clone_with_interceptor() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op.clone(), vec![interceptor]);
        let cloned = plan.clone();
        assert_eq!(cloned.len(), 1);
    }
}

#[cfg(test)]
mod invocation_plan_coverage_tests {
    use super::*;
    use crate::Operation;

    struct TestInterceptor;
    impl Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: std::sync::Arc<Invocation>, next: Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn invocation_plan_new_empty() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        assert_eq!(plan.operation().component(), "Service");
        assert_eq!(plan.operation().method(), "method");
        assert_eq!(plan.len(), 0);
        assert!(plan.is_empty());
    }

    #[test]
    fn invocation_plan_new_with_interceptor() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op, vec![interceptor]);
        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[test]
    fn invocation_plan_clone() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        let cloned = plan.clone();
        assert_eq!(cloned.len(), 0);
    }

    #[tokio::test]
    async fn invocation_plan_invoke_success() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op.clone(), vec![]);
        let invocation = std::sync::Arc::new(Invocation::new(op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invocation_plan_invoke_with_interceptor_success() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op.clone(), vec![interceptor]);
        let invocation = std::sync::Arc::new(Invocation::new(op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invocation_plan_invoke_mismatch_error() {
        let op = Operation::new("Service", "method");
        let plan = InvocationPlan::new(op, vec![]);
        let different_op = Operation::new("Other", "method");
        let invocation = std::sync::Arc::new(Invocation::new(different_op));
        let target: std::sync::Arc<crate::InvocationTarget> = std::sync::Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });
        let result = plan.invoke(invocation, target).await;
        assert!(result.is_err());
    }

    #[test]
    fn invocation_plan_clone_with_interceptor() {
        let op = Operation::new("Service", "method");
        let interceptor: std::sync::Arc<dyn Interceptor> = std::sync::Arc::new(TestInterceptor);
        let plan = InvocationPlan::new(op, vec![interceptor]);
        let cloned = plan.clone();
        assert_eq!(cloned.len(), 1);
    }
}
