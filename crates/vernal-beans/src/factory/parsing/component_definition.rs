//! ComponentDefinition — 对应 Spring beans.factory.parsing 中的组件定义概念。
//!
//! 此模块重新导出工厂级别的 [`crate::ComponentDefinition`]，同时提供
//! 解析阶段使用的轻量引用类型 [`ComponentDefinitionRef`]。
//!
//! 在 Spring 的 `beans.factory.parsing` 包中，`ComponentDefinition`
//! 描述一个 Bean 组件的元数据（名称、依赖、作用域等）。Vernal 的
//! 不可变组件定义定义在 `factory` 根模块，此处为解析子系统的便捷访问。

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
///
/// ## 初始化排序
///
/// 同一拓扑深度的组件按 `init_order` 升序排列（越小越早）。默认值
/// `i32::MAX` 表示最晚初始化。对标 tx_di 的 `init_sort` 模式。
pub struct ComponentDefinition {
    key: ComponentKey,
    dependencies: Vec<Dependency>,
    scope: Scope,
    factory: Arc<ErasedFactory>,
    /// 同层初始化排序值。同一拓扑深度的组件按此值升序排列。
    /// 默认 i32::MAX（最晚初始化）。
    init_order: i32,
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
            init_order: i32::MAX,
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
            init_order: i32::MAX,
        }
    }

    /// 为定义指定限定符。
    #[must_use]
    pub fn qualified(mut self, qualifier: Qualifier) -> Self {
        self.key = self.key.with_qualifier(qualifier);
        self
    }

    /// 设置同层初始化排序值。
    ///
    /// 同一拓扑深度的组件按此值升序排列（越小越早）。
    /// 默认值为 `i32::MAX`（最晚初始化）。
    ///
    /// # 参数
    /// - `order`：排序值，负数表示更早初始化
    #[must_use]
    pub fn with_init_order(mut self, order: i32) -> Self {
        self.init_order = order;
        self
    }

    /// 返回同层初始化排序值。
    #[must_use]
    pub fn init_order(&self) -> i32 {
        self.init_order
    }

    /// 声明一项无限定符依赖。
    #[must_use]
    pub fn depends_on<T: 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::of::<T>());
        self
    }

    /// 声明一项允许没有候选的立即具体类型依赖。
    ///
    /// 目标存在时仍进入 eager 依赖图；只有根候选不存在时工厂才会得到 `None`。
    #[must_use]
    pub fn depends_on_optional<T: 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::optional_of::<T>());
        self
    }

    /// 声明一个由 [`crate::ComponentProvider`] 延迟解析的具体类型依赖。
    ///
    /// Registry 会校验目标存在且唯一，但构建计划不会把它当作 eager 构造边。
    #[must_use]
    pub fn depends_on_provider<T: 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::provider_of::<T>());
        self
    }

    /// 声明一个允许目标不存在的延迟具体类型依赖。
    #[must_use]
    pub fn depends_on_optional_provider<T: 'static>(mut self) -> Self {
        self.dependencies
            .push(Dependency::optional_provider_of::<T>());
        self
    }

    /// 声明一项精确限定符依赖。
    #[must_use]
    pub fn depends_on_qualified<T: 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::qualified::<T>(qualifier));
        self
    }

    /// 声明一项允许没有精确限定符候选的立即具体类型依赖。
    #[must_use]
    pub fn depends_on_optional_qualified<T: 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::optional_qualified::<T>(qualifier));
        self
    }

    /// 声明一个带限定符的延迟具体类型依赖。
    #[must_use]
    pub fn depends_on_qualified_provider<T: 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::provider_qualified::<T>(qualifier));
        self
    }

    /// 声明一个允许目标不存在的带限定符延迟具体类型依赖。
    #[must_use]
    pub fn depends_on_optional_qualified_provider<T: 'static>(
        mut self,
        qualifier: Qualifier,
    ) -> Self {
        self.dependencies
            .push(Dependency::optional_provider_qualified::<T>(qualifier));
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

    /// 声明一项允许没有绑定的立即 Trait Object 依赖。
    ///
    /// 存在多个绑定时仍要求唯一候选或单一 Primary，不会静默选择首个实现。
    #[must_use]
    pub fn depends_on_optional_trait<T: ?Sized + 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::optional_trait_of::<T>());
        self
    }

    /// 声明一项带限定符的 Trait Object 依赖。
    #[must_use]
    pub fn depends_on_qualified_trait<T: ?Sized + 'static>(mut self, qualifier: Qualifier) -> Self {
        self.dependencies
            .push(Dependency::trait_qualified::<T>(qualifier));
        self
    }

    /// 声明一项允许没有精确限定符绑定的立即 Trait Object 依赖。
    #[must_use]
    pub fn depends_on_optional_qualified_trait<T: ?Sized + 'static>(
        mut self,
        qualifier: Qualifier,
    ) -> Self {
        self.dependencies
            .push(Dependency::optional_trait_qualified::<T>(qualifier));
        self
    }

    /// 声明由 [`crate::TraitProvider`] 延迟解析的唯一或 Primary Trait 实现。
    #[must_use]
    pub fn depends_on_trait_provider<T: ?Sized + 'static>(mut self) -> Self {
        self.dependencies.push(Dependency::trait_provider_of::<T>());
        self
    }

    /// 声明允许没有绑定的可选 Trait Provider。
    #[must_use]
    pub fn depends_on_optional_trait_provider<T: ?Sized + 'static>(mut self) -> Self {
        self.dependencies
            .push(Dependency::optional_trait_provider_of::<T>());
        self
    }

    /// 声明由 Trait Provider 延迟解析的精确命名实现。
    #[must_use]
    pub fn depends_on_qualified_trait_provider<T: ?Sized + 'static>(
        mut self,
        qualifier: Qualifier,
    ) -> Self {
        self.dependencies
            .push(Dependency::trait_provider_qualified::<T>(qualifier));
        self
    }

    /// 声明允许没有精确命名绑定的可选 Trait Provider。
    #[must_use]
    pub fn depends_on_optional_qualified_trait_provider<T: ?Sized + 'static>(
        mut self,
        qualifier: Qualifier,
    ) -> Self {
        self.dependencies
            .push(Dependency::optional_trait_provider_qualified::<T>(
                qualifier,
            ));
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

// ── 解析阶段使用的轻量引用类型 ──────────────────────────────────

/// 组件定义的轻量引用，用于事件传递时避免所有权转移。
///
/// 在 [`super::reader_event_listener::ReaderEventListener`] 的回调中使用，
/// 携带组件名称和类名信息供监听器读取。
#[derive(Debug, Clone)]
pub struct ComponentDefinitionRef {
    /// Bean 名称。
    bean_name: String,
    /// Bean 类名。
    bean_class_name: Option<String>,
}

impl ComponentDefinitionRef {
    /// 创建一个组件定义引用。
    pub fn new(bean_name: impl Into<String>, bean_class_name: Option<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean_class_name,
        }
    }

    /// 返回 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 返回 Bean 类名。
    pub fn bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }
}
