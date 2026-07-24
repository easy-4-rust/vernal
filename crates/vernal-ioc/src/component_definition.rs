//! 不可变组件定义对象。

use std::{any::Any, fmt, sync::Arc};

use vernal_core::BoxError;

use crate::{ComponentKey, Dependency, Qualifier, Resolver, Scope};

pub(crate) type ErasedComponent = Arc<dyn Any + Send + Sync>;
type ErasedFactory =
    dyn for<'a> Fn(&Resolver<'a>) -> Result<ErasedComponent, BoxError> + Send + Sync + 'static;

/// 描述一个组件的身份、依赖、作用域和构造方法。
///
/// 定义在注册阶段使用建造式 API 完成组装，进入 [`crate::Registry`] 后不再
/// 修改。组件工厂接收受限的 [`Resolver`]，只能访问这里显式声明的依赖。
pub struct ComponentDefinition {
    key: ComponentKey,
    dependencies: Vec<Dependency>,
    scope: Scope,
    factory: Arc<ErasedFactory>,
}

impl ComponentDefinition {
    /// 创建不会失败的单例组件定义。
    pub fn singleton<T, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        F: for<'a> Fn(&Resolver<'a>) -> T + Send + Sync + 'static,
    {
        Self::new::<T, _>(Scope::Singleton, move |resolver| Ok(factory(resolver)))
    }

    /// 创建可能返回业务错误的单例组件定义。
    pub fn try_singleton<T, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        F: for<'a> Fn(&Resolver<'a>) -> Result<T, BoxError> + Send + Sync + 'static,
    {
        Self::new(Scope::Singleton, factory)
    }

    /// 创建不会失败的瞬时组件定义。
    pub fn transient<T, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        F: for<'a> Fn(&Resolver<'a>) -> T + Send + Sync + 'static,
    {
        Self::new::<T, _>(Scope::Transient, move |resolver| Ok(factory(resolver)))
    }

    /// 创建可能返回业务错误的瞬时组件定义。
    pub fn try_transient<T, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        F: for<'a> Fn(&Resolver<'a>) -> Result<T, BoxError> + Send + Sync + 'static,
    {
        Self::new(Scope::Transient, factory)
    }

    /// 擦除具体组件类型，同时保留运行时安全的 `Any + Send + Sync` 边界。
    fn new<T, F>(scope: Scope, factory: F) -> Self
    where
        T: Any + Send + Sync,
        F: for<'a> Fn(&Resolver<'a>) -> Result<T, BoxError> + Send + Sync + 'static,
    {
        Self {
            key: ComponentKey::of::<T>(),
            dependencies: Vec::new(),
            scope,
            factory: Arc::new(move |resolver| {
                factory(resolver).map(|value| Arc::new(value) as ErasedComponent)
            }),
        }
    }

    /// 为定义指定限定符。
    #[must_use]
    pub fn qualified(mut self, qualifier: Qualifier) -> Self {
        self.key = self.key.with_qualifier(qualifier);
        self
    }

    /// 声明一项无限定符依赖。
    #[must_use]
    pub fn depends_on<T: 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::of::<T>());
        self
    }

    /// 声明一项精确限定符依赖。
    #[must_use]
    pub fn depends_on_qualified<T: 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::qualified::<T>(qualifier));
        self
    }

    /// 返回组件标识。
    #[must_use]
    pub fn key(&self) -> &ComponentKey {
        &self.key
    }

    /// 按声明顺序返回依赖选择器。
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// 返回组件作用域。
    #[must_use]
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// 调用擦除类型后的组件工厂。
    pub(crate) fn create(&self, resolver: &Resolver<'_>) -> Result<ErasedComponent, BoxError> {
        (self.factory)(resolver)
    }
}

impl fmt::Debug for ComponentDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentDefinition")
            .field("key", &self.key)
            .field("dependencies", &self.dependencies)
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}
