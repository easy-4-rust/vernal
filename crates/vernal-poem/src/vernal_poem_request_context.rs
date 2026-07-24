//! Poem Vernal 请求上下文提取器对象。

use std::sync::Arc;

use poem::{FromRequest, Request, RequestBody, Result};
use vernal_web::RequestContext;

use crate::PoemRejection;

/// 从严格 AOP Endpoint 写入的 Request Extensions 提取 `RequestContext`。
pub struct VernalPoemRequestContext(pub Arc<RequestContext>);

impl<'a> FromRequest<'a> for VernalPoemRequestContext {
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        request
            .extensions()
            .get::<Arc<RequestContext>>()
            .cloned()
            .map(Self)
            .ok_or_else(|| PoemRejection::MissingRequestContext.into())
    }
}
