//! Poem `IoC` 组件提取器对象。

use std::{any::Any, ops::Deref, sync::Arc};

use poem::{FromRequest, Request, RequestBody, Result};
use vernal_context::ApplicationContext;

use crate::PoemRejection;

/// 从当前 Poem 请求作用域解析类型化组件。
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
        let _context = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .ok_or(PoemRejection::MissingContext)?;
        let scope = request
            .extensions()
            .get::<Arc<vernal_web::WebRequestScope>>()
            .cloned()
            .ok_or(PoemRejection::MissingRequestScope)?;
        scope
            .resolve::<T>()
            .map(Self)
            .map_err(|error| PoemRejection::component_resolution(error).into())
    }
}
