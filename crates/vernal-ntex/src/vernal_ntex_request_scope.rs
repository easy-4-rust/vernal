//! Ntex 请求 Scope 提取器对象。

use std::sync::Arc;

use ntex::{
    http::Payload,
    web::{ErrorRenderer, FromRequest, HttpRequest},
};
use vernal_web::WebRequestScope;

use crate::NtexRejection;

/// 从 Ntex Request Extensions 提取当前 `WebRequestScope`。
pub struct VernalNtexRequestScope(pub Arc<WebRequestScope>);

impl<Err> FromRequest<Err> for VernalNtexRequestScope
where
    Err: ErrorRenderer,
{
    type Error = NtexRejection;

    async fn from_request(
        request: &HttpRequest,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Error> {
        request
            .extensions()
            .get::<Arc<WebRequestScope>>()
            .cloned()
            .map(Self)
            .ok_or(NtexRejection::MissingRequestScope)
    }
}
