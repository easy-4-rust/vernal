//! Axum 请求作用域提取器对象。

use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use vernal_web::WebRequestScope;

use crate::AxumRejection;

/// 从请求 Extension 提取当前 `WebRequestScope`。
pub struct VernalRequestScope(pub Arc<WebRequestScope>);

impl<S> FromRequestParts<S> for VernalRequestScope
where
    S: Send + Sync,
{
    type Rejection = AxumRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Arc<WebRequestScope>>()
            .cloned()
            .map(Self)
            .ok_or(AxumRejection::MissingRequestScope)
    }
}
