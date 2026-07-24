//! Ntex Vernal 请求上下文提取器对象。

use std::sync::Arc;

use ntex::{
    http::Payload,
    web::{ErrorRenderer, FromRequest, HttpRequest},
};
use vernal_web::RequestContext;

use crate::NtexRejection;

/// 从严格 Local-AOP 中间件写入的 Extensions 提取 `RequestContext`。
pub struct VernalNtexRequestContext(pub Arc<RequestContext>);

impl<Err> FromRequest<Err> for VernalNtexRequestContext
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
            .get::<Arc<RequestContext>>()
            .cloned()
            .map(Self)
            .ok_or(NtexRejection::MissingRequestContext)
    }
}
