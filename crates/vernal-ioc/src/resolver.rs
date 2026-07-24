//! 组件工厂受限解析器对象。

use std::{any::Any, sync::Arc};

use crate::{ComponentDefinition, ComponentKey, Container, Dependency, Qualifier, ResolveError};

/// 传递给组件工厂的受限依赖解析视图。
///
/// 解析器只允许读取当前 [`ComponentDefinition`] 已声明的依赖，使启动期验证过的
/// 依赖图保持权威，防止工厂退化成能够任意访问容器的 Service Locator。
pub struct Resolver<'a> {
    container: &'a Container,
    definition: &'a ComponentDefinition,
    stack: &'a [ComponentKey],
}

impl<'a> Resolver<'a> {
    /// 创建绑定到当前组件构造过程的解析器。
    pub(crate) fn new(
        container: &'a Container,
        definition: &'a ComponentDefinition,
        stack: &'a [ComponentKey],
    ) -> Self {
        Self {
            container,
            definition,
            stack,
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
            .resolve_all_traits_typed(&dependency, self.stack)
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
        self.container.resolve_typed(dependency, self.stack)
    }

    /// 校验声明后委托容器执行 Trait 单值解析。
    fn resolve_trait_dependency<T>(&self, dependency: &Dependency) -> Result<Arc<T>, ResolveError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.ensure_declared(dependency)?;
        self.container.resolve_trait_typed(dependency, self.stack)
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
