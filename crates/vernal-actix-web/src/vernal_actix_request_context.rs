//! Actix Vernal 请求上下文提取器对象。

use std::{future::Ready, sync::Arc};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use vernal_web::RequestContext;

use crate::ActixRejection;

/// 从严格 Local-AOP 中间件写入的 Extensions 提取 `RequestContext`。
pub struct VernalActixRequestContext(pub Arc<RequestContext>);

impl FromRequest for VernalActixRequestContext {
    type Error = ActixRejection;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let context = request
            .extensions()
            .get::<Arc<RequestContext>>()
            .cloned()
            .map(Self)
            .ok_or(ActixRejection::MissingRequestContext);
        std::future::ready(context)
    }
}
