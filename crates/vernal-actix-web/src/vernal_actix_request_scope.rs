//! Actix Web 请求 Scope 提取器对象。

use std::{future::Ready, sync::Arc};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use vernal_web::WebRequestScope;

use crate::ActixRejection;

/// 从 Actix Request Extensions 提取当前 `WebRequestScope`。
pub struct VernalActixRequestScope(pub Arc<WebRequestScope>);

impl FromRequest for VernalActixRequestScope {
    type Error = ActixRejection;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let scope = request
            .extensions()
            .get::<Arc<WebRequestScope>>()
            .cloned()
            .map(Self)
            .ok_or(ActixRejection::MissingRequestScope);
        std::future::ready(scope)
    }
}
