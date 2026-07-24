//! Axum 应用上下文提取器对象。

use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use vernal_context::ApplicationContext;

use crate::AxumRejection;

/// 从请求 Extension 提取显式 `ApplicationContext`。
pub struct VernalContext(pub Arc<ApplicationContext>);

impl<S> FromRequestParts<S> for VernalContext
where
    S: Send + Sync,
{
    type Rejection = AxumRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .map(Self)
            .ok_or(AxumRejection::MissingContext)
    }
}
