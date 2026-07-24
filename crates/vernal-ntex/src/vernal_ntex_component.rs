//! Ntex `IoC` 组件提取器对象。

use std::{any::Any, ops::Deref, sync::Arc};

use ntex::{
    http::Payload,
    web::{ErrorRenderer, FromRequest, HttpRequest},
};
use vernal_context::ApplicationContext;

use crate::NtexRejection;

/// 从当前 Ntex 请求作用域解析类型化组件。
pub struct VernalNtexComponent<T>(pub Arc<T>);

impl<T> Deref for VernalNtexComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, Err> FromRequest<Err> for VernalNtexComponent<T>
where
    T: Any + Send + Sync,
    Err: ErrorRenderer,
{
    type Error = NtexRejection;

    async fn from_request(
        request: &HttpRequest,
        _payload: &mut Payload,
    ) -> Result<Self, Self::Error> {
        let _context = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .or_else(|| request.app_state::<Arc<ApplicationContext>>().cloned())
            .ok_or(NtexRejection::MissingContext)?;
        let scope = request
            .extensions()
            .get::<Arc<vernal_web::WebRequestScope>>()
            .cloned()
            .ok_or(NtexRejection::MissingRequestScope)?;
        scope
            .resolve::<T>()
            .map(Self)
            .map_err(NtexRejection::component_resolution)
    }
}
