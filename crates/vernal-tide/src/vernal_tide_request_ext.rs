//! Tide Request 的 Vernal 类型化访问接口。

use std::{any::Any, sync::Arc};

use tide::Request;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::TideRejection;

/// 在 Tide 原生 `Request` 上读取 Context、组件与请求 Scope。
pub trait VernalTideRequestExt {
    /// 读取当前应用上下文。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalTideMiddleware`](crate::VernalTideMiddleware) 时返回拒绝。
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, TideRejection>;

    /// 从当前 Context 解析类型化 `IoC` 组件。
    ///
    /// # Errors
    ///
    /// Context 缺失或组件解析失败时返回结构化拒绝。
    fn vernal_component<T>(&self) -> Result<Arc<T>, TideRejection>
    where
        T: Any + Send + Sync;

    /// 读取当前请求作用域。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalTideMiddleware`](crate::VernalTideMiddleware) 时返回拒绝。
    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, TideRejection>;
}

impl<State> VernalTideRequestExt for Request<State> {
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, TideRejection> {
        self.ext::<Arc<ApplicationContext>>()
            .cloned()
            .ok_or(TideRejection::MissingContext)
    }

    fn vernal_component<T>(&self) -> Result<Arc<T>, TideRejection>
    where
        T: Any + Send + Sync,
    {
        self.vernal_context()?
            .container()
            .resolve::<T>()
            .map_err(TideRejection::component_resolution)
    }

    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, TideRejection> {
        self.ext::<Arc<WebRequestScope>>()
            .cloned()
            .ok_or(TideRejection::MissingRequestScope)
    }
}
