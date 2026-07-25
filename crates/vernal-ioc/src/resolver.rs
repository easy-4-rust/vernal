//! 组件工厂受限解析器对象。

use std::{any::Any, sync::Arc};

use crate::{
    ComponentDefinition, ComponentKey, ComponentProvider, Container, Dependency, Qualifier,
    ResolveError, ScopeContext, TraitProvider,
};

/// 传递给组件工厂的受限依赖解析视图。
///
/// 解析器只允许读取当前 [`ComponentDefinition`] 已声明的依赖，使启动期验证过的
/// 依赖图保持权威，防止工厂退化成能够任意访问容器的 Service Locator。
pub struct Resolver<'a> {
    container: &'a Container,
    definition: &'a ComponentDefinition,
    stack: &'a [ComponentKey],
    scope: Option<&'a ScopeContext>,
    construction_guard: &'a Arc<()>,
}

impl<'a> Resolver<'a> {
    /// 创建绑定到当前组件构造过程的解析器。
    pub(crate) fn new(
        container: &'a Container,
        definition: &'a ComponentDefinition,
        stack: &'a [ComponentKey],
        scope: Option<&'a ScopeContext>,
        construction_guard: &'a Arc<()>,
    ) -> Self {
        Self {
            container,
            definition,
            stack,
            scope,
            construction_guard,
        }
    }

    /// 解析一项已声明的无限定符依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明、无法唯一选择或构造失败时返回 [`ResolveError`]。
    pub fn resolve<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_dependency(&Dependency::of::<T>())
    }

    /// 解析一项已声明的精确限定符依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明、无法选择或构造失败时返回 [`ResolveError`]。
    pub fn resolve_qualified<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_dependency(&Dependency::qualified::<T>(qualifier.clone()))
    }

    /// 解析一项允许没有候选的立即具体类型依赖。
    ///
    /// 只有根候选不存在时返回 `Ok(None)`；候选歧义、构造失败、类型恢复和 Scope
    /// 错误仍保持结构化失败。
    ///
    /// # Errors
    ///
    /// 依赖未声明，或已存在候选无法完成选择、构造和类型恢复时返回
    /// [`ResolveError`]。
    pub fn resolve_optional<T>(&self) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_optional_dependency(&Dependency::optional_of::<T>())
    }

    /// 解析一项允许没有精确限定符候选的立即具体类型依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明，或匹配候选存在但解析失败时返回 [`ResolveError`]。
    pub fn resolve_optional_qualified<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.resolve_optional_dependency(&Dependency::optional_qualified::<T>(qualifier.clone()))
    }

    /// 创建一个延迟解析唯一具体类型依赖的 Provider。
    ///
    /// Provider 只持有这里校验过的选择器，后续不能请求其他类型。
    ///
    /// # Errors
    ///
    /// 当前定义没有声明对应 Provider 依赖时返回 [`ResolveError`]。
    pub fn provider<T>(&self) -> Result<ComponentProvider<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.create_provider(Dependency::provider_of::<T>())
    }

    /// 创建一个带限定符的延迟具体类型 Provider。
    ///
    /// # Errors
    ///
    /// 当前定义没有声明完全相同的限定符 Provider 时返回 [`ResolveError`]。
    pub fn qualified_provider<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<ComponentProvider<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.create_provider(Dependency::provider_qualified::<T>(qualifier.clone()))
    }

    /// 创建允许没有候选定义的可选 Provider。
    ///
    /// # Errors
    ///
    /// 当前定义没有声明可选 Provider 时返回 [`ResolveError`]。
    pub fn optional_provider<T>(&self) -> Result<ComponentProvider<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.create_provider(Dependency::optional_provider_of::<T>())
    }

    /// 创建允许没有精确限定符候选的可选 Provider。
    ///
    /// # Errors
    ///
    /// 当前定义没有声明完全相同的可选限定符 Provider 时返回 [`ResolveError`]。
    pub fn optional_qualified_provider<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<ComponentProvider<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.create_provider(Dependency::optional_provider_qualified::<T>(
            qualifier.clone(),
        ))
    }

    /// 创建延迟解析唯一或 Primary Trait 实现的受限 Provider。
    ///
    /// # Errors
    ///
    /// 当前组件没有声明对应 Trait Provider 依赖时返回 [`ResolveError`]。
    pub fn trait_provider<T>(&self) -> Result<TraitProvider<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.create_trait_provider(Dependency::trait_provider_of::<T>())
    }

    /// 创建延迟解析精确命名 Trait 实现的受限 Provider。
    ///
    /// # Errors
    ///
    /// 当前组件没有声明相同 qualifier 的 Trait Provider 时返回 [`ResolveError`]。
    pub fn qualified_trait_provider<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<TraitProvider<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.create_trait_provider(Dependency::trait_provider_qualified::<T>(qualifier.clone()))
    }

    /// 创建允许没有 Trait Binding 的可选 Provider。
    ///
    /// # Errors
    ///
    /// 当前组件没有声明可选 Trait Provider 时返回 [`ResolveError`]。
    pub fn optional_trait_provider<T>(&self) -> Result<TraitProvider<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.create_trait_provider(Dependency::optional_trait_provider_of::<T>())
    }

    /// 创建允许没有精确命名绑定的可选 Trait Provider。
    ///
    /// # Errors
    ///
    /// 当前组件没有声明相同 qualifier 的可选 Trait Provider 时返回
    /// [`ResolveError`]。
    pub fn optional_qualified_trait_provider<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<TraitProvider<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.create_trait_provider(Dependency::optional_trait_provider_qualified::<T>(
            qualifier.clone(),
        ))
    }

    /// 解析一项已声明的 Trait Object 单值依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明、没有绑定、绑定歧义或目标构造失败时返回 [`ResolveError`]。
    pub fn resolve_trait<T>(&self) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_trait_dependency(&Dependency::trait_of::<T>())
    }

    /// 解析一项已声明的带限定符 Trait Object 依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明、没有精确绑定或目标构造失败时返回 [`ResolveError`]。
    pub fn resolve_qualified_trait<T>(&self, qualifier: &Qualifier) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_trait_dependency(&Dependency::trait_qualified::<T>(qualifier.clone()))
    }

    /// 解析一项允许没有绑定的立即 Trait Object 依赖。
    ///
    /// 零绑定返回 `Ok(None)`；多个绑定仍要求唯一候选或单一 Primary。
    ///
    /// # Errors
    ///
    /// 依赖未声明、绑定歧义、目标构造或 Trait 投影失败时返回 [`ResolveError`]。
    pub fn resolve_optional_trait<T>(&self) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_optional_trait_dependency(&Dependency::optional_trait_of::<T>())
    }

    /// 解析一项允许没有精确限定符绑定的立即 Trait Object 依赖。
    ///
    /// # Errors
    ///
    /// 依赖未声明，或匹配绑定存在但解析失败时返回 [`ResolveError`]。
    pub fn resolve_optional_qualified_trait<T>(
        &self,
        qualifier: &Qualifier,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.resolve_optional_trait_dependency(&Dependency::optional_trait_qualified::<T>(
            qualifier.clone(),
        ))
    }

    /// 解析一项已声明 Trait Object 的全部实现。
    ///
    /// 没有绑定时返回空集合；绑定目标的构造或转换失败仍返回结构化错误。
    ///
    /// # Errors
    ///
    /// 依赖未声明、目标构造失败或绑定转换失败时返回 [`ResolveError`]。
    pub fn resolve_all_traits<T>(&self) -> Result<Vec<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let dependency = Dependency::all_traits_of::<T>();
        self.ensure_declared(&dependency)?;
        self.container
            .resolve_all_traits_typed(&dependency, self.stack, self.scope)
    }

    /// 返回当前正在构造的组件标识。
    #[must_use]
    pub fn component(&self) -> &ComponentKey {
        self.definition.key()
    }

    /// 校验依赖声明后委托容器执行解析。
    fn resolve_dependency<T>(&self, dependency: &Dependency) -> Result<Arc<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.ensure_declared(dependency)?;
        self.container
            .resolve_typed(dependency, self.stack, self.scope)
    }

    /// 校验声明后执行立即具体类型的可选解析。
    fn resolve_optional_dependency<T>(
        &self,
        dependency: &Dependency,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.ensure_declared(dependency)?;
        self.container
            .resolve_optional_typed(dependency, self.stack, self.scope)
    }

    /// 校验声明后委托容器执行 Trait 单值解析。
    fn resolve_trait_dependency<T>(&self, dependency: &Dependency) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_declared(dependency)?;
        self.container
            .resolve_trait_typed(dependency, self.stack, self.scope)
    }

    /// 校验声明后执行立即 Trait Object 的可选解析。
    fn resolve_optional_trait_dependency<T>(
        &self,
        dependency: &Dependency,
    ) -> Result<Option<Arc<T>>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_declared(dependency)?;
        self.container
            .resolve_optional_trait_typed(dependency, self.stack, self.scope)
    }

    /// 校验 Provider 元数据并创建共享当前 Container 身份的受限句柄。
    fn create_provider<T>(
        &self,
        dependency: Dependency,
    ) -> Result<ComponentProvider<T>, ResolveError>
    where
        T: Any + Send + Sync,
    {
        self.ensure_declared(&dependency)?;
        Ok(ComponentProvider::new(
            self.container,
            self.definition.key().clone(),
            dependency,
            Arc::downgrade(self.construction_guard),
        ))
    }

    /// 校验 Trait Provider 元数据并创建共享当前 Container 的受限句柄。
    fn create_trait_provider<T>(
        &self,
        dependency: Dependency,
    ) -> Result<TraitProvider<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_declared(&dependency)?;
        Ok(TraitProvider::new(
            self.container,
            self.definition.key().clone(),
            dependency,
            Arc::downgrade(self.construction_guard),
        ))
    }

    /// 确认工厂请求的依赖已经进入不可变组件定义。
    fn ensure_declared(&self, dependency: &Dependency) -> Result<(), ResolveError> {
        if self.definition.dependencies().contains(dependency) {
            return Ok(());
        }
        Err(ResolveError::UndeclaredDependency {
            component: self.definition.key().clone(),
            dependency: dependency.to_string(),
        })
    }
}
