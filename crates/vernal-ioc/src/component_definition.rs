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
    /// 将一个已经构造完成的 Rust 原生对象注册为共享实例。
    ///
    /// 该方法适合注册由生态框架创建、Vernal 只负责注入的对象，例如 Tokio
    /// [`tokio::runtime::Handle`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html)、
    /// HTTP 客户端、数据库连接池或 Web 框架状态。对象不需要实现 Vernal
    /// 专用 trait，只需要满足 `Any + Send + Sync`。
    ///
    /// 与 [`Self::singleton`] 的“每个容器调用一次工厂”语义不同，预构建实例
    /// 会被创建自同一注册表的所有容器共享。需要容器级隔离时应继续使用
    /// [`Self::singleton`]。
    pub fn shared_value<T>(value: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self::shared_arc(Arc::new(value))
    }

    /// 将调用方已经持有的 `Arc<T>` 注册为共享实例。
    ///
    /// 容器解析结果仍是同一个 `Arc<T>`，不会形成 `Arc<Arc<T>>`。这使框架
    /// 原生共享状态可以直接进入依赖图，同时保留调用方原有的所有权关系。
    pub fn shared_arc<T>(value: Arc<T>) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            key: ComponentKey::of::<T>(),
            dependencies: Vec::new(),
            scope: Scope::Singleton,
            factory: Arc::new(move |_| Ok(Arc::clone(&value) as ErasedComponent)),
        }
    }

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

    /// 创建不会失败的类型化自定义作用域组件定义。
    ///
    /// 标记类型 `S` 只提供作用域身份，不需要实现 Vernal trait。组件必须通过
    /// [`crate::Container::resolve_in`] 在匹配的 [`crate::ScopeContext`] 中解析。
    pub fn scoped<T, S, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        S: 'static,
        F: for<'a> Fn(&Resolver<'a>) -> T + Send + Sync + 'static,
    {
        Self::new::<T, _>(Scope::custom::<S>(), move |resolver| Ok(factory(resolver)))
    }

    /// 创建可能返回业务错误的类型化自定义作用域组件定义。
    pub fn try_scoped<T, S, F>(factory: F) -> Self
    where
        T: Any + Send + Sync,
        S: 'static,
        F: for<'a> Fn(&Resolver<'a>) -> Result<T, BoxError> + Send + Sync + 'static,
    {
        Self::new(Scope::custom::<S>(), factory)
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

    /// 声明一项 Trait Object 单值依赖。
    ///
    /// 无限定符选择要求只有一个绑定，或多个绑定中恰好一个标记为 Primary。
    #[must_use]
    pub fn depends_on_trait<T: ?Sized + 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::trait_of::<T>());
        self
    }

    /// 声明一项带限定符的 Trait Object 依赖。
    #[must_use]
    pub fn depends_on_qualified_trait<T: ?Sized + 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::trait_qualified::<T>(qualifier));
        self
    }

    /// 声明指定 Trait Object 的全部实现依赖。
    ///
    /// 没有任何绑定时工厂会得到空集合；存在绑定时所有目标都会进入依赖图。
    #[must_use]
    pub fn depends_on_all_traits<T: ?Sized + 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::all_traits_of::<T>());
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
