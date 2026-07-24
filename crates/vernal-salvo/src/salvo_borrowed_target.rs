//! Salvo Hoop 借用型 Send-AOP 目标对象。

use std::{mem, sync::Arc};

use salvo::{Depot, FlowCtrl, Request, Response};
use vernal_aop::{
    BorrowedInvocationTarget, Invocation, InvocationError, InvocationFuture, InvocationValue,
};

use crate::SalvoResponse;

/// 在一次 Hoop 调用期内独占借用 Salvo 请求、响应与流程控制器。
pub(crate) struct SalvoBorrowedTarget<'a> {
    request: &'a mut Request,
    depot: &'a mut Depot,
    response: &'a mut Response,
    control: &'a mut FlowCtrl,
    envelope: Arc<SalvoResponse>,
}

impl<'a> SalvoBorrowedTarget<'a> {
    /// 创建只在当前 `Handler::handle` Future 内有效的最终目标。
    pub(crate) fn new(
        request: &'a mut Request,
        depot: &'a mut Depot,
        response: &'a mut Response,
        control: &'a mut FlowCtrl,
        envelope: Arc<SalvoResponse>,
    ) -> Self {
        Self {
            request,
            depot,
            response,
            control,
            envelope,
        }
    }
}

impl BorrowedInvocationTarget for SalvoBorrowedTarget<'_> {
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        Box::pin(async move {
            self.control
                .call_next(self.request, self.depot, self.response)
                .await;
            self.envelope
                .store(mem::take(self.response))
                .await
                .map_err(InvocationError::target)?;
            Ok(Box::new(Arc::clone(&self.envelope)) as InvocationValue)
        })
    }
}
