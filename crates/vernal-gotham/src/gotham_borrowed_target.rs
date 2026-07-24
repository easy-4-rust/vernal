//! Gotham Pipeline 借用型 Send-AOP 目标对象。

use std::{pin::Pin, sync::Arc};

use gotham::{
    handler::{HandlerError, HandlerFuture},
    state::State,
};
use vernal_aop::{
    BorrowedInvocationTarget, Invocation, InvocationError, InvocationFuture, InvocationValue,
};

use crate::{GothamAopError, GothamResponse, gotham_upstream_error::GothamUpstreamError};

/// 在一次 Middleware 调用期内拥有 State 与只能消费一次的 Pipeline Chain。
pub(crate) struct GothamBorrowedTarget<Chain> {
    state: Option<State>,
    chain: Option<Chain>,
    completed_state: Option<State>,
    native_error: Option<(State, HandlerError)>,
    response: Arc<GothamResponse>,
}

impl<Chain> GothamBorrowedTarget<Chain> {
    /// 创建只在当前 Gotham Middleware Future 内有效的最终目标。
    pub(crate) fn new(state: State, chain: Chain, response: Arc<GothamResponse>) -> Self {
        Self {
            state: Some(state),
            chain: Some(chain),
            completed_state: None,
            native_error: None,
            response,
        }
    }

    /// 取回策略短路或成功执行后仍属于当前请求的 State。
    pub(crate) fn take_state(&mut self) -> Option<State> {
        self.state
            .take()
            .or_else(|| self.completed_state.take())
            .or_else(|| self.native_error.take().map(|(state, _)| state))
    }

    /// 取回未经转换的 Gotham Handler 原生错误。
    pub(crate) fn take_native_error(&mut self) -> Option<(State, HandlerError)> {
        self.native_error.take()
    }
}

impl<Chain> BorrowedInvocationTarget for GothamBorrowedTarget<Chain>
where
    Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
{
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        Box::pin(async move {
            let state = self
                .state
                .take()
                .ok_or_else(|| InvocationError::target(GothamAopError::StateAlreadyTaken))?;
            let chain = self
                .chain
                .take()
                .ok_or_else(|| InvocationError::target(GothamAopError::ChainAlreadyTaken))?;

            match chain(state).await {
                Ok((state, response)) => {
                    self.completed_state = Some(state);
                    self.response
                        .store(response)
                        .await
                        .map_err(InvocationError::target)?;
                    Ok(Box::new(Arc::clone(&self.response)) as InvocationValue)
                }
                Err((state, error)) => {
                    self.native_error = Some((state, error));
                    Err(InvocationError::target(GothamUpstreamError))
                }
            }
        })
    }
}
