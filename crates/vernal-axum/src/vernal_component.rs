//! Axum `IoC` 组件提取器对象。

use std::{any::Any, ops::Deref, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use vernal_context::ApplicationContext;

use crate::AxumRejection;

/// 从当前请求作用域解析类型化 `IoC` 组件。
///
/// 组件遵循 Vernal 中注册的 Singleton、Transient 或 `WebRequestScope` 自定义
/// 作用域语义；提取器不会创建第二个容器，也不会使用进程级全局注册表。
pub struct VernalComponent<T>(pub Arc<T>);

impl<T> Deref for VernalComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S, T> FromRequestParts<S> for VernalComponent<T>
where
    S: Send + Sync,
    T: Any + Send + Sync,
{
    type Rejection = AxumRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let _context = parts
            .extensions
            .get::<Arc<ApplicationContext>>()
            .ok_or(AxumRejection::MissingContext)?;
        let scope = parts
            .extensions
            .get::<Arc<vernal_web::WebRequestScope>>()
            .ok_or(AxumRejection::MissingRequestScope)?;
        scope
            .resolve::<T>()
            .map(Self)
            .map_err(AxumRejection::component_resolution)
    }
}
