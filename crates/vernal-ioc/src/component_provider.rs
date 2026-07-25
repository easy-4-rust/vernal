//! 类型安全延迟组件提供者对象。

use std::{
    any::Any,
    fmt,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{ComponentKey, Container, Dependency, ResolveError, ScopeContext};

/// 只允许按一个已声明类型选择器访问当前 Container 的延迟组件提供者。
///
/// Provider 适合让长生命周期组件按调用取得新的 Transient、显式访问当前请求/
/// 任务 Scope，或表达可选扩展点。它不暴露任意类型查询能力，因此不会退化成全局
/// Service Locator；目标类型、限定符和可选语义都必须进入组件定义并在构建期校验。
pub struct ComponentProvider<T> {
    container: Container,
    consumer: ComponentKey,
    dependency: Dependency,
    construction_guard: Weak<()>,
    marker: PhantomData<fn() -> T>,
}

impl<T> ComponentProvider<T>
where
    T: Any + Send + Sync,
{
    /// 使用当前 Container 的共享实例状态创建受限 Provider。
    pub(crate) fn new(
        container: &Container,
        consumer: ComponentKey,
        dependency: Dependency,
        construction_guard: Weak<()>,
    ) -> Self {
        Self {
            container: container.shared_handle(),
            consumer,
            dependency,
            construction_guard,
            marker: PhantomData,
        }
    }

    /// 解析目标组件；Transient 每次创建新实例，Singleton 复用当前 Container 缓存。
    ///
    /// # Errors
    ///
    /// 目标缺失、候选歧义、需要未激活 Scope 或组件构造失败时返回
    /// [`ResolveError`]。
    pub fn get(&self) -> Result<Arc<T>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container.resolve_typed(&self.dependency, &[], None)
    }

    /// 在显式 Scope 链中解析目标组件。
    ///
    /// 该入口允许 Singleton 持有 Provider，而不捕获某个具体请求 Scope；调用方必须
    /// 在每次业务调用时传入当前 Scope。
    ///
    /// # Errors
    ///
    /// Scope 属于其他 Container、已经关闭、目标 Scope 未激活，或组件构造失败时
    /// 返回 [`ResolveError`]。
    pub fn get_in(&self, scope: &ScopeContext) -> Result<Arc<T>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container.ensure_scope_owner(scope)?;
        self.container
            .resolve_typed(&self.dependency, &[], Some(scope))
    }

    /// 目标定义不存在时返回 `None`，存在时保持与 [`Self::get`] 相同的解析语义。
    ///
    /// 该方法主要配合 `#[component(optional)]`。歧义、构造失败和作用域错误不会被
    /// 降级成 `None`。
    ///
    /// # Errors
    ///
    /// 候选歧义、组件构造或类型恢复失败时返回 [`ResolveError`]。
    pub fn get_if_available(&self) -> Result<Option<Arc<T>>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container
            .resolve_optional_typed(&self.dependency, &[], None)
    }

    /// 在显式 Scope 中可选解析目标定义。
    ///
    /// # Errors
    ///
    /// Scope 身份、候选歧义、组件构造或类型恢复失败时返回 [`ResolveError`]。
    pub fn get_if_available_in(
        &self,
        scope: &ScopeContext,
    ) -> Result<Option<Arc<T>>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container.ensure_scope_owner(scope)?;
        self.container
            .resolve_optional_typed(&self.dependency, &[], Some(scope))
    }

    /// 返回 Provider 是否由可选依赖声明创建。
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.dependency.is_optional()
    }

    /// 拒绝在消费方工厂返回前重入 Provider。
    fn ensure_consumer_constructed(&self) -> Result<(), ResolveError> {
        if self.construction_guard.upgrade().is_none() {
            return Ok(());
        }
        Err(ResolveError::ProviderUsedDuringConstruction {
            component: self.consumer.clone(),
            dependency: self.dependency.to_string(),
        })
    }
}

impl<T> Clone for ComponentProvider<T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container.shared_handle(),
            consumer: self.consumer.clone(),
            dependency: self.dependency.clone(),
            construction_guard: self.construction_guard.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ComponentProvider<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentProvider")
            .field("consumer", &self.consumer)
            .field("dependency", &self.dependency)
            .finish_non_exhaustive()
    }
}
