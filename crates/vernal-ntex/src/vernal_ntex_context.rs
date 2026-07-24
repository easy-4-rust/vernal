//! Ntex 应用上下文提取器对象。

use std::sync::Arc;

use ntex::{
    http::Payload,
    web::{ErrorRenderer, FromRequest, HttpRequest},
};
use vernal_context::ApplicationContext;

use crate::NtexRejection;

/// 从请求扩展或 Ntex App State 提取显式 `ApplicationContext`。
pub struct VernalNtexContext(pub Arc<ApplicationContext>);

impl<Err> FromRequest<Err> for VernalNtexContext
where
    Err: ErrorRenderer,
{
    type Error = NtexRejection;

    async fn from_request(
        request: &HttpRequest,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Error> {
        let context = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .or_else(|| request.app_state::<Arc<ApplicationContext>>().cloned());
        context.map(Self).ok_or(NtexRejection::MissingContext)
    }
}
