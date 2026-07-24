//! Axum 请求上下文提取器对象。

use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use vernal_web::RequestContext;

use crate::AxumRejection;

/// 提取由严格 AOP Layer 创建或传播的 `RequestContext`。
pub struct VernalRequestContext(pub Arc<RequestContext>);

impl<S> FromRequestParts<S> for VernalRequestContext
where
    S: Send + Sync,
{
    type Rejection = AxumRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Arc<RequestContext>>()
            .cloned()
            .map(Self)
            .ok_or(AxumRejection::MissingRequestContext)
    }
}
