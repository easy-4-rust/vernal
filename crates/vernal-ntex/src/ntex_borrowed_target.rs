//! Ntex Pipeline 借用型 Local-AOP 目标对象。

use std::{cell::RefCell, fmt, rc::Rc, sync::Arc};

use ntex::{
    service::{Service, ServiceCtx},
    web::{ErrorRenderer, WebRequest, WebResponse},
};
use vernal_aop::{
    BorrowedLocalInvocationTarget, Invocation, LocalInvocationError, LocalInvocationFuture,
    LocalInvocationValue,
};

use crate::{
    NtexAopError, VernalNtexService, ntex_response_envelope::NtexResponseEnvelope,
    ntex_service_error::NtexServiceError,
};

/// 在一次 Ntex Pipeline 调用期内借用 Service 与 `ServiceCtx` 的最终目标。
pub(crate) struct NtexBorrowedTarget<'a, S, Err> {
    service: &'a S,
    service_context: ServiceCtx<'a, VernalNtexService<S>>,
    request: Rc<RefCell<Option<WebRequest<Err>>>>,
    response: Rc<NtexResponseEnvelope>,
}

impl<'a, S, Err> NtexBorrowedTarget<'a, S, Err> {
    /// 创建只在当前 Worker 调用期内有效的目标对象。
    pub(crate) fn new(
        service: &'a S,
        service_context: ServiceCtx<'a, VernalNtexService<S>>,
        request: Rc<RefCell<Option<WebRequest<Err>>>>,
        response: Rc<NtexResponseEnvelope>,
    ) -> Self {
        Self {
            service,
            service_context,
            request,
            response,
        }
    }
}

impl<S, Err> BorrowedLocalInvocationTarget for NtexBorrowedTarget<'_, S, Err>
where
    S: Service<WebRequest<Err>, Response = WebResponse>,
    S::Error: fmt::Debug + fmt::Display + 'static,
    Err: ErrorRenderer,
{
    fn invoke(&self, _invocation: Arc<Invocation>) -> LocalInvocationFuture<'_> {
        Box::pin(async move {
            // 一次性信封在调用下游前立即释放 RefCell 可变借用，绝不让 Guard
            // 跨越 await，也不会克隆会破坏 Ntex Payload 唯一性的 HttpRequest。
            let request =
                self.request.borrow_mut().take().ok_or_else(|| {
                    LocalInvocationError::target(NtexAopError::RequestAlreadyTaken)
                })?;
            let response = self
                .service_context
                .call(self.service, request)
                .await
                .map_err(|error| LocalInvocationError::target(NtexServiceError::new(error)))?;
            self.response
                .store(response)
                .map_err(LocalInvocationError::target)?;
            Ok(Box::new(Rc::clone(&self.response)) as LocalInvocationValue)
        })
    }
}
