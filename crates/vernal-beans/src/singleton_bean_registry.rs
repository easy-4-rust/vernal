//! SingletonBeanRegistry — Spring 风格的单例 Bean 注册表接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.SingletonBeanRegistry`。
//!
//! 提供单例 Bean 的注册、查询和管理能力。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的单例 Bean 注册表接口。
///
/// 对应 Spring 的 `SingletonBeanRegistry`。
///
/// 提供单例 Bean 的注册、查询和管理能力。
/// `Container` 内部的 `singletons` 字段实现了此接口的语义。
///
/// ## 方法
///
/// - `register_singleton` — 注册单例实例
/// - `get_singleton` — 获取单例实例
/// - `contains_singleton` — 检查是否包含
/// - `singleton_names` — 列举所有名称
/// - `singleton_count` — 单例总数
/// - `register_singleton_callback` — 注册单例创建回调
pub trait SingletonBeanRegistry: Send + Sync + 'static {
    /// 注册单例实例。
    ///
    /// 对应 Spring 的 `void registerSingleton(String beanName, Object singletonObject)`。
    fn register_singleton(&self, bean_name: &str, singleton_object: Arc<dyn Any + Send + Sync>);

    /// 获取单例实例。
    ///
    /// 对应 Spring 的 `Object getSingleton(String beanName)`。
    fn get_singleton(&self, bean_name: &str) -> Option<Arc<dyn Any + Send + Sync>>;

    /// 检查是否包含指定单例。
    ///
    /// 对应 Spring 的 `boolean containsSingleton(String beanName)`。
    fn contains_singleton(&self, bean_name: &str) -> bool;

    /// 获取所有单例名称。
    ///
    /// 对应 Spring 的 `String[] getSingletonNames()`。
    fn singleton_names(&self) -> Vec<String>;

    /// 单例总数。
    ///
    /// 对应 Spring 的 `int getSingletonCount()`。
    fn singleton_count(&self) -> usize;

    /// 注册单例创建回调。
    ///
    /// 对应 Spring 的 `void addSingletonCallback(String beanName, Consumer<Object> singletonConsumer)`。
    ///
    /// 当指定 Bean 的单例实例被创建后，调用此回调。
    fn add_singleton_callback(
        &mut self,
        bean_name: String,
        callback: Arc<dyn Fn(&dyn Any) + Send + Sync>,
    );

    /// 获取单例互斥锁。
    ///
    /// 对应 Spring 的 `Object getSingletonMutex()`（6.2 已废弃）。
    ///
    /// 返回一个 `Arc` 引用，可用于外部同步。
    fn singleton_mutex(&self) -> Arc<dyn Any + Send + Sync>;
}
