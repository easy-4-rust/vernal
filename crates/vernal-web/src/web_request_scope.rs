//! Web 请求级组件作用域对象。

use std::{any::Any, error::Error, future::Future, sync::Arc};

use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_ioc::{
    Container, Qualifier, Registry, ResolveError, ScopeContext, ScopeError, ScopeKey, ScopeState,
};

use crate::web_request_scope_owner::WebRequestScopeOwner;

/// 将 `IoC` 自定义作用域投影为框架中立的单请求组件作用域。
///
/// `WebRequestScope` 本身同时充当 `ScopeKey` 的标记类型，因此业务组件可以直接
/// 声明 `#[component(scope = WebRequestScope)]`。正常 Adapter 必须通过
/// [`Self::from_application_context`] 创建它；这样组件提取器、AOP 调用上下文和
/// 响应 Body 释放逻辑共享同一个 [`ScopeContext`]，不会再维护第二套缓存或状态机。
///
/// 为兼容只使用类型缓存的低层调用，[`Self::new`] 会创建绑定空注册表的独立
/// Container。该模式可以缓存原生请求对象和执行关闭钩子，但不能解析应用注册的
/// 业务组件；框架集成不应使用它。
pub struct WebRequestScope {
    owner: WebRequestScopeOwner,
    scope: Arc<ScopeContext>,
}

impl WebRequestScope {
    /// 创建不绑定 `ApplicationContext` 的兼容请求作用域。
    ///
    /// 调用方已有应用上下文时应使用 [`Self::from_application_context`]，否则
    /// [`Self::resolve`] 只能从空注册表返回组件缺失错误。
    #[must_use]
    pub fn new(cancellation: CancellationToken) -> Self {
        let container = Container::new(Registry::empty());
        let scope = container.open_scope_with_cancellation::<Self>(cancellation);
        Self {
            owner: WebRequestScopeOwner::Standalone(container),
            scope,
        }
    }

    /// 基于真实应用上下文创建请求作用域。
    ///
    /// 作用域取消令牌是应用取消树的子令牌：请求结束只取消当前请求，应用关闭则
    /// 会立即阻止所有存活请求继续创建作用域组件。
    #[must_use]
    pub fn from_application_context(context: Arc<ApplicationContext>) -> Self {
        let scope = context.open_scope::<Self>();
        Self {
            owner: WebRequestScopeOwner::Application(context),
            scope,
        }
    }

    /// 在当前请求作用域解析唯一注册的 `T` 组件。
    ///
    /// Singleton、Transient 与 `WebRequestScope` 自定义组件均遵循 `IoC` 内核的
    /// 同一依赖图和生命周期规则。
    ///
    /// # Errors
    ///
    /// 组件缺失、歧义、构造失败或作用域已经关闭时返回 [`ResolveError`]。
    pub fn resolve<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.owner.container().resolve_in(&self.scope)
    }

    /// 在当前请求作用域解析带限定符的 `T` 组件。
    ///
    /// # Errors
    ///
    /// 限定符不存在、构造失败或作用域不可用时返回 [`ResolveError`]。
    pub fn resolve_qualified<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.owner
            .container()
            .resolve_qualified_in(qualifier, &self.scope)
    }

    /// 在当前请求作用域解析 Trait Object 的唯一或 Primary 实现。
    ///
    /// # Errors
    ///
    /// Trait Binding 缺失、存在歧义、目标构造失败或作用域不可用时返回
    /// [`ResolveError`]。
    pub fn resolve_trait<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.owner.container().resolve_trait_in(&self.scope)
    }

    /// 在当前请求作用域解析带限定符的 Trait Object。
    ///
    /// # Errors
    ///
    /// 命名绑定缺失、目标构造失败或作用域不可用时返回 [`ResolveError`]。
    pub fn resolve_qualified_trait<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.owner
            .container()
            .resolve_qualified_trait_in(qualifier, &self.scope)
    }

    /// 在当前请求作用域解析某个 Trait 的全部实现。
    ///
    /// # Errors
    ///
    /// 任一实现构造失败、类型转换失败或作用域不可用时返回 [`ResolveError`]。
    pub fn resolve_all_traits<T>(&self) -> Result<Vec<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.owner.container().resolve_all_traits_in(&self.scope)
    }

    /// 获取或惰性创建一种框架原生请求对象。
    ///
    /// 该入口与 `IoC` 请求组件共享 `ScopeContext` 生命周期，但使用独立缓存
    /// 命名空间；即使原生对象与业务组件类型相同也不会互相覆盖。对于已注册业务
    /// 组件应优先使用 [`Self::resolve`]，以保留依赖图和结构化构造错误。
    ///
    /// # Errors
    ///
    /// Scope 已开始关闭、取消或缓存类型不一致时返回 [`ScopeError`]。
    pub fn get_or_insert_with<T, F>(&self, factory: F) -> Result<Arc<T>, ScopeError>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> T,
    {
        self.scope.get_or_insert_with(factory)
    }

    /// 注册一个逆序执行的异步关闭钩子。
    ///
    /// # Errors
    ///
    /// Scope 已开始关闭或应用取消树已取消时返回 [`ScopeError`]。
    pub fn on_close<F, Fut, E>(&self, hook: F) -> Result<(), ScopeError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        self.scope.on_close(hook)
    }

    /// 取消请求、等待既有构造结束并逆序执行全部关闭钩子。
    ///
    /// # Errors
    ///
    /// 返回钩子失败、钩子任务异常、Runtime 缺失或等待超时等结构化错误；后台
    /// 协调器仍执行其余钩子、清空缓存并进入 Closed。应用绑定 Scope 发生错误时
    /// 还会向所属 `ApplicationContext` 写入静态、
    /// 脱敏的 `web.request-scope.cleanup-failed` 告警代码；独立兼容 Scope
    /// 没有应用诊断目标，因此只返回原始结构化错误。应用绑定 Scope 自动采用
    /// [`vernal_context::ScopeCleanupPolicy`]；等待超时不会取消后台关闭任务。
    pub async fn close(&self) -> Result<(), ScopeError> {
        let application = self.owner.application_context().cloned();
        let result = match application
            .as_ref()
            .and_then(|context| context.scope_cleanup_policy().timeout())
        {
            Some(maximum_wait) => self.scope.close_with_timeout(maximum_wait).await,
            None => self.scope.close().await,
        };
        if result.is_err() {
            if let Some(context) = application {
                context
                    .record_runtime_warning("web.request-scope.cleanup-failed")
                    .await;
            }
        }
        result
    }

    /// 返回当前请求作用域状态快照。
    pub fn state(&self) -> ScopeState {
        self.scope.state()
    }

    /// 返回请求作用域的类型化身份。
    #[must_use]
    pub fn key(&self) -> ScopeKey {
        self.scope.key()
    }

    /// 返回底层 `IoC` 自定义作用域。
    ///
    /// 集成层需要建立子作用域或调用更低层容器 API 时可以显式使用该引用；普通
    /// Handler 应优先使用本对象提供的解析方法。
    #[must_use]
    pub const fn scope_context(&self) -> &Arc<ScopeContext> {
        &self.scope
    }

    /// 返回请求所属的真实应用上下文。
    ///
    /// 兼容模式 [`Self::new`] 没有应用上下文，因此返回 `None`。
    #[must_use]
    pub fn application_context(&self) -> Option<&Arc<ApplicationContext>> {
        self.owner.application_context()
    }

    /// 返回请求取消令牌。
    #[must_use]
    pub fn cancellation(&self) -> &CancellationToken {
        self.scope.cancellation()
    }
}
