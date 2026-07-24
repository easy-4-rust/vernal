//! Salvo Depot 的 Vernal 类型化访问接口。

use std::{any::Any, sync::Arc};

use salvo::Depot;
use vernal_context::ApplicationContext;
use vernal_web::{RequestContext, WebRequestScope};

use crate::SalvoRejection;

/// 在 Salvo 原生 `Depot` 上读取 Context、组件与请求 Scope。
pub trait VernalSalvoDepotExt {
    /// 读取当前应用上下文。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalSalvoHoop`](crate::VernalSalvoHoop) 时返回脱敏拒绝。
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, SalvoRejection>;

    /// 从当前请求作用域解析类型化 `IoC` 组件。
    ///
    /// # Errors
    ///
    /// Context 缺失或组件解析失败时返回结构化拒绝。
    fn vernal_component<T>(&self) -> Result<Arc<T>, SalvoRejection>
    where
        T: Any + Send + Sync;

    /// 读取当前请求作用域。
    ///
    /// # Errors
    ///
    /// 未安装 [`VernalSalvoHoop`](crate::VernalSalvoHoop) 时返回脱敏拒绝。
    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, SalvoRejection>;

    /// 读取严格 AOP 为当前请求创建的类型化请求上下文。
    ///
    /// # Errors
    ///
    /// 使用普通 Hoop，或严格调用尚未建立上下文时返回脱敏拒绝。
    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, SalvoRejection>;
}

impl VernalSalvoDepotExt for Depot {
    fn vernal_context(&self) -> Result<Arc<ApplicationContext>, SalvoRejection> {
        self.obtain::<Arc<ApplicationContext>>()
            .cloned()
            .map_err(|_| SalvoRejection::MissingContext)
    }

    fn vernal_component<T>(&self) -> Result<Arc<T>, SalvoRejection>
    where
        T: Any + Send + Sync,
    {
        let _context = self.vernal_context()?;
        self.vernal_request_scope()?
            .resolve::<T>()
            .map_err(SalvoRejection::component_resolution)
    }

    fn vernal_request_scope(&self) -> Result<Arc<WebRequestScope>, SalvoRejection> {
        self.obtain::<Arc<WebRequestScope>>()
            .cloned()
            .map_err(|_| SalvoRejection::MissingRequestScope)
    }

    fn vernal_request_context(&self) -> Result<Arc<RequestContext>, SalvoRejection> {
        self.obtain::<Arc<RequestContext>>()
            .cloned()
            .map_err(|_| SalvoRejection::MissingRequestContext)
    }
}
