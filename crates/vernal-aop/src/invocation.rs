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
    context: InvocationContext,
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
            context: InvocationContext::new(),
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
    pub const fn context(&self) -> &InvocationContext {
        &self.context
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
}
