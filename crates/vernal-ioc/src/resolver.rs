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
        if !self.definition.dependencies().contains(dependency) {
            return Err(ResolveError::UndeclaredDependency {
                component: self.definition.key().clone(),
                dependency: dependency.to_string(),
            });
        }
        self.container.resolve_typed(dependency, self.stack)
    }
}
