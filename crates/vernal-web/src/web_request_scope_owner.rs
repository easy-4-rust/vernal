//! Web 请求作用域所有者对象。

use std::sync::Arc;

use vernal_context::ApplicationContext;
use vernal_ioc::Container;

/// 保持请求作用域所绑定的组件容器在整个响应 Body 生命周期内有效。
///
/// 正常 Adapter 使用 `Application` 分支，因而能解析应用注册表中的 Singleton、
/// Transient 和自定义请求作用域组件。`Standalone` 只为兼容不经过
/// `ApplicationContext` 的底层调用保留，其空容器仍与同一个 `ScopeContext`
/// 共享类型缓存和关闭状态，但不会凭空发现业务组件。
pub(crate) enum WebRequestScopeOwner {
    /// 由真实应用上下文拥有。
    Application(Arc<ApplicationContext>),
    /// 由空注册表容器独立拥有。
    Standalone(Container),
}

impl WebRequestScopeOwner {
    /// 返回当前请求作用域唯一绑定的组件容器。
    pub(crate) fn container(&self) -> &Container {
        match self {
            Self::Application(context) => context.container(),
            Self::Standalone(container) => container,
        }
    }

    /// 返回可选的真实应用上下文。
    pub(crate) fn application_context(&self) -> Option<&Arc<ApplicationContext>> {
        match self {
            Self::Application(context) => Some(context),
            Self::Standalone(_) => None,
        }
    }
}
