//! Tonic Request 扩展对象。

use std::{any::Any, sync::Arc};

use tonic::{GrpcMethod, Request};
use vernal_context::ApplicationContext;
use vernal_web::{RequestContext, RouteMetadata, WebRequestScope};

use crate::TonicRequestError;

/// 从 Tonic 原生 Request Extensions 获取 Vernal 能力。
///
/// 所有方法都只读取当前请求携带的显式对象，不访问全局容器。Unary 与 Streaming
/// Handler 使用相同的 `tonic::Request<T>`，因此无需额外的响应式抽象。
pub trait TonicRequestExt {
    /// 返回请求携带的应用上下文。
    ///
    /// # Errors
    ///
    /// 未安装 `VernalLayer` 或 `TonicContextInterceptor` 时返回 Internal Status。
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, TonicRequestError>;

    /// 从当前 Tower 请求作用域解析类型化组件。
    ///
    /// # Errors
    ///
    /// 上下文缺失或 `IoC` 解析失败时返回结构化请求错误。
    fn vernal_component<C>(&self) -> Result<Arc<C>, TonicRequestError>
    where
        C: Any + Send + Sync;

    /// 返回 Tower 请求作用域层写入的请求 Scope。
    ///
    /// # Errors
    ///
    /// 未安装 `RequestScopeLayer` 时返回 Internal Status。
    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, TonicRequestError>;

    /// 返回严格 AOP Layer 创建或传播的请求上下文。
    ///
    /// # Errors
    ///
    /// 请求没有经过 `TonicAopLayer` 时返回 Internal Status。
    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, TonicRequestError>;

    /// 从 Tonic 的 `GrpcMethod` Extension 构建稳定路由元数据。
    ///
    /// # Errors
    ///
    /// Request 不含 `GrpcMethod` 时返回 Internal Status。
    fn vernal_route_metadata(&self) -> Result<RouteMetadata, TonicRequestError>;
}

impl<T> TonicRequestExt for Request<T> {
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, TonicRequestError> {
        self.extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
            .ok_or(TonicRequestError::MissingContext)
    }

    fn vernal_component<C>(&self) -> Result<Arc<C>, TonicRequestError>
    where
        C: Any + Send + Sync,
    {
        let _context = self.vernal_context()?;
        self.vernal_request_scope()?
            .resolve::<C>()
            .map_err(TonicRequestError::component_resolution)
    }

    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, TonicRequestError> {
        self.extensions()
            .get::<Arc<WebRequestScope>>()
            .cloned()
            .ok_or(TonicRequestError::MissingRequestScope)
    }

    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, TonicRequestError> {
        self.extensions()
            .get::<Arc<RequestContext>>()
            .cloned()
            .ok_or(TonicRequestError::MissingRequestContext)
    }

    fn vernal_route_metadata(&self) -> Result<RouteMetadata, TonicRequestError> {
        let method = self
            .extensions()
            .get::<GrpcMethod<'static>>()
            .ok_or(TonicRequestError::MissingGrpcMethod)?;
        Ok(RouteMetadata::new(
            method.service(),
            method.method(),
            format!("/{}/{}", method.service(), method.method()),
        ))
    }
}
