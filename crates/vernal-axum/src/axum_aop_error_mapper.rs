//! Axum AOP Service 错误映射策略对象。

use std::convert::Infallible;

use axum::response::{IntoResponse, Response};
use vernal_tower::{AopServiceError, TowerErrorMapper};

use crate::AxumAopError;

/// 把 Tower AOP 结构化错误恢复成 Axum 原生响应。
///
/// Axum Router 的下游错误类型是 `Infallible`，因此严格 AOP 的基础设施失败和
/// 策略短路都应在此协议边界转换成响应；状态码和脱敏正文继续由
/// [`AxumAopError`] 决定，通用 `vernal-tower` 不感知 HTTP 策略。
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AxumAopErrorMapper;

impl TowerErrorMapper for AxumAopErrorMapper {
    type Source = AopServiceError<Infallible>;
    type Response = Response;
    type Error = Infallible;

    fn map_error(&self, error: Self::Source) -> Result<Self::Response, Self::Error> {
        Ok(AxumAopError::new(error).into_response())
    }
}
