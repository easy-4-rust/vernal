//! Poem 请求 Scope 提取器对象。

use std::sync::Arc;

use poem::{FromRequest, Request, RequestBody, Result};
use vernal_web::WebRequestScope;

use crate::PoemRejection;

/// 从 Poem Request Extensions 提取当前 `WebRequestScope`。
pub struct VernalPoemRequestScope(pub Arc<WebRequestScope>);

impl<'a> FromRequest<'a> for VernalPoemRequestScope {
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        request
            .extensions()
            .get::<Arc<WebRequestScope>>()
            .cloned()
            .map(Self)
            .ok_or_else(|| PoemRejection::MissingRequestScope.into())
    }
}
