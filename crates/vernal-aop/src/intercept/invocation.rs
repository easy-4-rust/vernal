//! 运行时调用对象。

use std::sync::Arc;

use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::{InvocationContext, InvocationId, Operation};

/// 一次被拦截调用的不可变身份与共享运行时状态。
///
/// 调用对象由 `Arc` 在拦截器之间传递。截止时间采用 Tokio `Instant`，取消采用
/// `CancellationToken`，从而可以自然接入 Tokio task、Web 请求断开和应用关闭。
pub struct Invocation {
    id: InvocationId,
    operation: Operation,
    context: Arc<InvocationContext>,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
}

impl Invocation {
    /// 创建没有截止时间的新调用。
    #[must_use]
    pub fn new(operation: Operation) -> Self {
        Self {
            id: InvocationId::next(),
            operation,
            context: Arc::new(InvocationContext::new()),
            cancellation: CancellationToken::new(),
            deadline: None,
        }
    }

    /// 使用调用方提供的取消令牌。
    #[must_use]
    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }

    /// 设置绝对截止时间。
    #[must_use]
    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// 转换成便于在异步调用链中共享的 `Arc`。
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// 返回调用标识。
    #[must_use]
    pub const fn id(&self) -> InvocationId {
        self.id
    }

    /// 返回操作描述。
    #[must_use]
    pub fn operation(&self) -> &Operation {
        &self.operation
    }

    /// 返回强类型扩展上下文。
    #[must_use]
    pub fn context(&self) -> &InvocationContext {
        self.context.as_ref()
    }

    /// 返回取消令牌。
    #[must_use]
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 返回可选绝对截止时间。
    #[must_use]
    pub const fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    /// 使用计划中的权威声明操作创建同一次调用的运行视图。
    ///
    /// 新对象保留 Invocation ID、强类型 Context、取消令牌和 deadline，只替换
    /// Operation 的声明元数据。运行期 Adapter 因而只需提供稳定身份，拦截器和
    /// 目标仍能读取应用启动阶段验证过的标签与限定符。
    pub(crate) fn for_declared_operation(&self, operation: Operation) -> Arc<Self> {
        Arc::new(Self {
            id: self.id,
            operation,
            context: Arc::clone(&self.context),
            cancellation: self.cancellation.clone(),
            deadline: self.deadline,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn invocation_new() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        assert_eq!(inv.operation().component(), "Service");
    }

    #[test]
    fn invocation_operation() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        assert_eq!(inv.operation().method(), "method");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn invocation_id() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        let _id = inv.id();
    }

    #[test]
    fn invocation_context() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        let _ctx = inv.context();
    }

    #[test]
    fn invocation_cancellation() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        let _token = inv.cancellation();
    }

    #[test]
    fn invocation_deadline_none() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        assert!(inv.deadline().is_none());
    }

    #[test]
    fn invocation_with_cancellation() {
        let op = Operation::new("Service", "method");
        let token = CancellationToken::new();
        let inv = Invocation::new(op).with_cancellation(token);
        assert!(!inv.cancellation().is_cancelled());
    }

    #[test]
    fn invocation_with_deadline() {
        let op = Operation::new("Service", "method");
        let deadline = Instant::now() + std::time::Duration::from_secs(60);
        let inv = Invocation::new(op).with_deadline(deadline);
        assert!(inv.deadline().is_some());
    }

    #[test]
    fn invocation_shared() {
        let op = Operation::new("Service", "method");
        let inv = Invocation::new(op);
        let shared = inv.shared();
        assert_eq!(shared.operation().component(), "Service");
    }

    #[test]
    fn invocation_for_declared_operation() {
        let op1 = Operation::new("Service", "method1");
        let op2 = Operation::new("Service", "method2");
        let inv = Invocation::new(op1);
        let new_inv = inv.for_declared_operation(op2);
        assert_eq!(new_inv.operation().method(), "method2");
    }
}
