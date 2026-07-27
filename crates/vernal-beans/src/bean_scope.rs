//! Scope trait — Spring 风格的自定义作用域接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.Scope`。
//!
//! Spring 的 `Scope` 接口定义了 Bean 在某种上下文（Request/Session/Application）
//! 中的实例缓存策略。vernal-beans 提供此 trait 作为作用域扩展点，具体实现
//! 由 `vernal-context`（Request/Session/Application）或用户自定义。

use std::any::Any;

/// Spring 风格的自定义作用域接口。
///
/// 对应 Spring 的 `Scope` 接口：每个 Scope 代表一种 Bean 缓存策略。
/// 与 `vernal-beans::scope::Scope` 枚举（内建 Singleton/Transient/Custom）不同，
/// 此 trait 是扩展接口——用于注册自定义 Scope 实现。
///
/// ## 方法
///
/// - `get` — 获取或创建作用域内的 Bean 实例
/// - `remove` — 移除并返回作用域内的 Bean 实例
/// - `register_destruction_callback` — 注册销毁回调
/// - `resolve_contextual_object` — 解析上下文对象
/// - `conversation_id` — 返回会话标识
///
/// ## 与 vernal 现有 Scope 的关系
///
/// `vernal-beans::scope::Scope` 是枚举类型（Singleton/Transient/Custom），用于
/// 在 `ComponentDefinition` 中声明作用域。此 `BeanScope` trait 是运行时扩展接口，
/// 通过 `Container::register_scope(name, scope)` 注册，用于支持自定义 Scope 的
/// `get` / `remove` / 销毁回调。
pub trait BeanScope: Send + Sync + 'static {
    /// 获取或创建作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `Scope.get(String name, ObjectFactory objectFactory)`。
    /// `object_factory` 是工厂闭包，首次调用时用于创建实例；后续调用返回缓存。
    fn get(
        &self,
        name: &str,
        object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 移除并返回作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `Scope.remove(String name)`。
    /// 如果该作用域支持主动移除 Bean，此方法将删除并返回实例。
    /// 不支持移除时返回 `Ok(None)`。
    fn remove(
        &self,
        name: &str,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// 注册销毁回调。
    ///
    /// 对应 Spring 的 `Scope.registerDestructionCallback(String name, Runnable callback)`。
    /// 作用域关闭时，按注册顺序执行所有回调。
    fn register_destruction_callback(
        &self,
        name: &str,
        callback: Box<dyn FnOnce() + Send + Sync>,
    ) {
        let _ = (name, callback);
    }

    /// 解析上下文对象。
    ///
    /// 对应 Spring 的 `Scope.resolveContextualObject(String key)`。
    /// 例如 RequestScope 返回 `ServletRequest`，SessionScope 返回 `HttpSession`。
    fn resolve_contextual_object(&self, _key: &str) -> Option<Box<dyn Any>> {
        None
    }

    /// 返回会话标识。
    ///
    /// 对应 Spring 的 `Scope.getConversationId()`。
    fn conversation_id(&self) -> Option<&str> {
        None
    }
}
