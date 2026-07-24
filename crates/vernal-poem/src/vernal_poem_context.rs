//! Poem 应用上下文提取器对象。

use std::sync::Arc;

use poem::{FromRequest, Request, RequestBody, Result};
use vernal_context::ApplicationContext;

use crate::PoemRejection;

/// 从 Poem Request Extensions 提取当前 `ApplicationContext`。
pub struct VernalPoemContext(pub Arc<ApplicationContext>);

impl<'a> FromRequest<'a> for VernalPoemContext {
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .map(Self)
            .ok_or_else(|| PoemRejection::MissingContext.into())
    }
}
