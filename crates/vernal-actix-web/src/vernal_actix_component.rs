//! Actix Web `IoC` 组件提取器对象。

use std::{any::Any, future::Ready, ops::Deref, sync::Arc};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload, web};
use vernal_context::ApplicationContext;

use crate::ActixRejection;

/// 从当前 `ApplicationContext` 解析类型化组件。
pub struct VernalActixComponent<T>(pub Arc<T>);

impl<T> Deref for VernalActixComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for VernalActixComponent<T>
where
    T: Any + Send + Sync,
{
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
        let result = context
            .ok_or(ActixRejection::MissingContext)
            .and_then(|context| {
                context
                    .container()
                    .resolve::<T>()
                    .map(Self)
                    .map_err(ActixRejection::component_resolution)
            });
        std::future::ready(result)
    }
}
