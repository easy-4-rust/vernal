//! ObjectProvider — Spring 风格的对象供应器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.ObjectProvider`。
//!
//! 延迟/可选的 Bean 供应器，提供安全的 Bean 获取方式。

use std::sync::Arc;

/// Spring 风格的对象供应器接口。
///
/// 对应 Spring 的 `ObjectProvider<T>`。
///
/// 提供延迟/可选/安全的 Bean 获取方式：
/// - `get()` — 获取实例（延迟创建）
/// - `if_available` — 获取可用实例（不存在时返回 None）
/// - `stream` — 获取所有实例的流
/// - `ordered_stream` — 按顺序获取流
///
/// ## 与 vernal-beans 现有体系的关系
///
/// `ComponentProvider` 是 vernal-beans 现有的类型安全 Provider。
/// `ObjectProvider` 是 Spring 风格的接口，更接近 `ObjectProvider<T>` 的 API。
///
/// ## 设计
///
/// 使用 `dyn ObjectProvider<T>` 而非具体类型，允许 `Container` 在运行时
/// 返回不同实现（按类型查询、按名称查询等）。
pub trait ObjectProvider<T: ?Sized + Send + Sync>: Send + Sync {
    /// 获取实例（延迟创建）。
    ///
    /// 对应 Spring 的 `T getObject() throws BeansException`。
    ///
    /// # 错误
    ///
    /// Bean 不存在或创建失败时返回 `Err`。
    fn get(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取可用实例（不存在时返回 None）。
    ///
    /// 对应 Spring 的 `default T ifAvailable()`。
    ///
    /// 如果 Bean 不存在或创建失败，返回 `Ok(None)` 而非 `Err`。
    fn if_available(&self) -> Option<Arc<T>>;

    /// 获取实例（不存在时抛出异常）。
    ///
    /// 对应 Spring 的 `default T getIfUnique()`。
    ///
    /// 如果存在多个实现，抛出 `NoUniqueBeanDefinitionException`。
    fn get_if_unique(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取所有实例的流。
    ///
    /// 对应 Spring 的 `default Stream<T> stream()`。
    fn stream(&self) -> Vec<Arc<T>>;

    /// 增量获取（消费一次，下次从头开始）。
    ///
    /// 对应 Spring 的 `default Stream<T> orderedStream()`。
    fn ordered_stream(&self) -> Vec<Arc<T>>;
}
