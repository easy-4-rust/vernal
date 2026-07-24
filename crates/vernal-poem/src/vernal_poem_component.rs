//! Poem `IoC` 组件提取器对象。

use std::{any::Any, ops::Deref, sync::Arc};

use poem::{FromRequest, Request, RequestBody, Result};
use vernal_context::ApplicationContext;

use crate::PoemRejection;

/// 从当前 `ApplicationContext` 解析类型化组件。
pub struct VernalPoemComponent<T>(pub Arc<T>);

impl<T> Deref for VernalPoemComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> FromRequest<'a> for VernalPoemComponent<T>
where
    T: Any + Send + Sync,
{
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let context = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .ok_or(PoemRejection::MissingContext)?;
        context
            .container()
            .resolve::<T>()
            .map(Self)
            .map_err(|error| PoemRejection::component_resolution(error).into())
    }
}
