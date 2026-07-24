//! Rocket Handler 借用型 Send-AOP 目标对象。

use std::sync::Arc;

use rocket::{
    Data, Request,
    route::{Handler, Outcome},
};
use vernal_aop::{
    BorrowedInvocationTarget, Invocation, InvocationError, InvocationFuture, InvocationValue,
};

use crate::{RocketAopError, rocket_outcome_marker::RocketOutcomeMarker};

/// 在一次 Route 调用内借用 Request/Handler，并独占拥有一次性 Data。
pub(crate) struct RocketBorrowedTarget<'r, 'q, 'h> {
    handler: &'h dyn Handler,
    request: &'r Request<'q>,
    data: Option<Data<'r>>,
    outcome: Option<Outcome<'r>>,
}

impl<'r, 'q, 'h> RocketBorrowedTarget<'r, 'q, 'h> {
    /// 创建只在当前 Rocket Handler Future 内有效的最终目标。
    pub(crate) fn new(handler: &'h dyn Handler, request: &'r Request<'q>, data: Data<'r>) -> Self {
        Self {
            handler,
            request,
            data: Some(data),
            outcome: None,
        }
    }

    /// 取回 Handler 产生的原生三态 Outcome。
    pub(crate) fn take_outcome(&mut self) -> Option<Outcome<'r>> {
        self.outcome.take()
    }
}

impl BorrowedInvocationTarget for RocketBorrowedTarget<'_, '_, '_> {
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        Box::pin(async move {
            let data = self
                .data
                .take()
                .ok_or_else(|| InvocationError::target(RocketAopError::DataAlreadyTaken))?;
            self.outcome = Some(self.handler.handle(self.request, data).await);
            Ok(Box::new(Arc::new(RocketOutcomeMarker)) as InvocationValue)
        })
    }
}
