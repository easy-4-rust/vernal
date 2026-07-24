//! Actix Web 应用上下文提取器对象。

use std::{future::Ready, sync::Arc};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload, web};
use vernal_context::ApplicationContext;

use crate::ActixRejection;

/// 从 Request Extensions 或 App Data 提取显式 `ApplicationContext`。
pub struct VernalActixContext(pub Arc<ApplicationContext>);

impl FromRequest for VernalActixContext {
    type Error = ActixRejection;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let context = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .or_else(|| {
                request
                    .app_data::<web::Data<ApplicationContext>>()
                    .map(|data| data.clone().into_inner())
            });
        std::future::ready(context.map(Self).ok_or(ActixRejection::MissingContext))
    }
}
