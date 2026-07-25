//! 类型安全 Trait Object 延迟提供者对象。

use std::{
    fmt,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{ComponentKey, Container, Dependency, ResolveError, ScopeContext};

/// 只允许按一个已声明 Trait Binding 选择器访问当前 Container 的延迟提供者。
///
/// Rust 无法用稳定泛型特化让具体类型与 unsized Trait Object 共用一套恢复逻辑，
/// 因此本对象与 [`crate::ComponentProvider`] 分离。它只保存固定 Trait 类型、
/// qualifier 与 optional 语义，不能查询其他类型，也不会建立全局 Service Locator。
pub struct TraitProvider<T: ?Sized> {
    container: Container,
    consumer: ComponentKey,
    dependency: Dependency,
    construction_guard: Weak<()>,
    marker: PhantomData<fn(&T)>,
}

impl<T> TraitProvider<T>
where
    T: ?Sized + Send + Sync + 'static,
{
    /// 使用当前 Container 的共享状态创建受限 Trait Provider。
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

    /// 解析唯一、Primary 或精确命名 Trait 实现。
    ///
    /// # Errors
    ///
    /// 绑定缺失/歧义、目标构造失败、作用域未激活或投影类型不匹配时返回
    /// [`ResolveError`]。
    pub fn get(&self) -> Result<Arc<T>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container
            .resolve_trait_typed(&self.dependency, &[], None)
    }

    /// 在调用方显式传入的当前 Scope 链中解析 Trait 实现。
    ///
    /// # Errors
    ///
    /// Scope 来自其他 Container、目标 Scope 不可用，或绑定解析失败时返回
    /// [`ResolveError`]。
    pub fn get_in(&self, scope: &ScopeContext) -> Result<Arc<T>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container.ensure_scope_owner(scope)?;
        self.container
            .resolve_trait_typed(&self.dependency, &[], Some(scope))
    }

    /// 根 Trait Binding 不存在时返回 `None`，其他错误保持可见。
    ///
    /// # Errors
    ///
    /// 候选歧义、目标构造、作用域或 Trait 投影失败时返回 [`ResolveError`]。
    pub fn get_if_available(&self) -> Result<Option<Arc<T>>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container
            .resolve_optional_trait_typed(&self.dependency, &[], None)
    }

    /// 在显式 Scope 中可选解析 Trait 实现。
    ///
    /// # Errors
    ///
    /// Scope 身份、候选歧义、目标构造或 Trait 投影失败时返回 [`ResolveError`]。
    pub fn get_if_available_in(
        &self,
        scope: &ScopeContext,
    ) -> Result<Option<Arc<T>>, ResolveError> {
        self.ensure_consumer_constructed()?;
        self.container.ensure_scope_owner(scope)?;
        self.container
            .resolve_optional_trait_typed(&self.dependency, &[], Some(scope))
    }

    /// 返回 Provider 是否允许没有 Trait Binding。
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.dependency.is_optional()
    }

    /// 拒绝在消费方工厂返回前重入 Trait Provider。
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

impl<T: ?Sized> Clone for TraitProvider<T> {
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

impl<T: ?Sized> fmt::Debug for TraitProvider<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TraitProvider")
            .field("consumer", &self.consumer)
            .field("dependency", &self.dependency)
            .finish_non_exhaustive()
    }
}
