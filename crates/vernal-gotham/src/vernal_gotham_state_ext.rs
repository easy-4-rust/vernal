//! Gotham State 的 Vernal 类型化访问接口。

use std::{any::Any, sync::Arc};

use gotham::state::State;
use vernal_context::ApplicationContext;
use vernal_web::{RequestContext, WebRequestScope};

use crate::{
    GothamRejection, VernalGothamContext, VernalGothamRequestContext, VernalGothamRequestScope,
};

/// 在 Gotham 原生 `State` 上读取 Context、组件与请求 Scope。
pub trait VernalGothamStateExt {
    /// 读取当前应用上下文。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalGothamMiddleware`](crate::VernalGothamMiddleware) 时返回拒绝。
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, GothamRejection>;

    /// 从当前 Context 解析类型化 `IoC` 组件。
    ///
    /// # Errors
    ///
    /// Context 缺失或组件解析失败时返回结构化拒绝。
    fn vernal_component<T>(&self) -> Result<Arc<T>, GothamRejection>
    where
        T: Any + Send + Sync;

    /// 读取当前请求作用域。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalGothamMiddleware`](crate::VernalGothamMiddleware) 时返回拒绝。
    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, GothamRejection>;

    /// 读取严格 AOP 创建的请求上下文。
    ///
    /// # Errors
    ///
    /// 使用普通中间件，或严格调用尚未建立上下文时返回结构化拒绝。
    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, GothamRejection>;
}

impl VernalGothamStateExt for State {
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, GothamRejection> {
        self.try_borrow::<VernalGothamContext>()
            .map(|context| Arc::clone(&context.0))
            .ok_or(GothamRejection::MissingContext)
    }

    fn vernal_component<T>(&self) -> Result<Arc<T>, GothamRejection>
    where
        T: Any + Send + Sync,
    {
        self.vernal_context()?
            .container()
            .resolve::<T>()
            .map_err(GothamRejection::component_resolution)
    }

    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, GothamRejection> {
        self.try_borrow::<VernalGothamRequestScope>()
            .map(|scope| Arc::clone(&scope.0))
            .ok_or(GothamRejection::MissingRequestScope)
    }

    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, GothamRejection> {
        self.try_borrow::<VernalGothamRequestContext>()
            .map(|context| Arc::clone(&context.0))
            .ok_or(GothamRejection::MissingRequestContext)
    }
}
