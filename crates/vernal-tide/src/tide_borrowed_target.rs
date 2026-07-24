//! Tide `Next` 借用型 Send-AOP 目标对象。

use std::sync::Arc;

use tide::{Next, Request};
use vernal_aop::{
    BorrowedInvocationTarget, Invocation, InvocationError, InvocationFuture, InvocationValue,
};

use crate::{TideAopError, TideResponse};

/// 在一次 Middleware 调用期内拥有 Request，并借用路由内部 `Next`。
pub(crate) struct TideBorrowedTarget<'a, State> {
    request: Option<Request<State>>,
    next: Option<Next<'a, State>>,
    response: Arc<TideResponse>,
}

impl<'a, State> TideBorrowedTarget<'a, State> {
    /// 创建只在当前 Middleware Future 内有效的最终目标。
    pub(crate) fn new(
        request: Request<State>,
        next: Next<'a, State>,
        response: Arc<TideResponse>,
    ) -> Self {
        Self {
            request: Some(request),
            next: Some(next),
            response,
        }
    }
}

impl<State> BorrowedInvocationTarget for TideBorrowedTarget<'_, State>
where
    State: Clone + Send + Sync + 'static,
{
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        Box::pin(async move {
            let request = self
                .request
                .take()
                .ok_or_else(|| InvocationError::target(TideAopError::RequestAlreadyTaken))?;
            let next = self
                .next
                .take()
                .ok_or_else(|| InvocationError::target(TideAopError::NextAlreadyTaken))?;
            self.response
                .store(next.run(request).await)
                .await
                .map_err(InvocationError::target)?;
            Ok(Box::new(Arc::clone(&self.response)) as InvocationValue)
        })
    }
}
