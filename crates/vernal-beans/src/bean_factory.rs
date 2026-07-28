//! BeanFactory — Spring 风格的 IoC 容器顶层接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanFactory`。
//!
//! 这是访问 Spring 风格 IoC 容器的基础接口，提供按名称/类型获取 Bean 实例的能力。
//! `Container` 是此接口的主要实现。

use std::any::Any;
use std::sync::Arc;

use crate::component_key::ComponentKey;
use crate::object_provider::ObjectProvider;

/// FactoryBean 前缀：`"&"`。
///
/// 当 Bean 名称以 `&` 开头时，返回的是 FactoryBean 本身而非其 `getObject()` 的结果。
/// 对应 Spring 的 `BeanFactory.FACTORY_BEAN_PREFIX`。
pub const FACTORY_BEAN_PREFIX: &str = "&";

/// Spring 风格的 IoC 容器顶层接口。
///
/// 对应 Spring 的 `BeanFactory`。
///
/// 这是访问 Spring 风格 IoC 容器的基础客户端视图。进一步的接口
/// 如 `ListableBeanFactory` 和 `ConfigurableBeanFactory` 可用于特定目的。
///
/// ## 主要方法
///
/// - `get_bean` — 按名称获取 Bean
/// - `get_bean_by_type_id` — 按类型获取 Bean
/// - `contains_bean` — 检查是否包含 Bean
/// - `is_singleton` / `is_prototype` — 查询作用域
/// - `get_type` — 获取 Bean 类型
/// - `get_aliases` — 获取别名
/// - `get_bean_provider` — 获取延迟/可选 Bean 供应器
///
/// ## 设计（dyn 兼容）
///
/// 为保持 `dyn BeanFactory` 可用，所有方法使用 `TypeId` 而非泛型参数。
pub trait BeanFactory: Send + Sync + 'static {
    /// 按完整组件标识获取实例。
    ///
    /// 对应 Spring 的 `Object getBean(String name) throws BeansException`。
    fn get_bean_by_key(
        &self,
        key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 按类型获取唯一实例。
    ///
    /// 对应 Spring 的 `<T> T getBean(Class<T> requiredType) throws BeansException`。
    /// 使用 TypeId 替代泛型以保持 dyn 兼容。
    fn get_bean_by_type_id(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 检查容器是否包含指定 Bean。
    ///
    /// 对应 Spring 的 `boolean containsBean(String name)`。
    fn contains_bean(&self, key: &ComponentKey) -> bool;

    /// 查询 Bean 是否为 singleton。
    ///
    /// 对应 Spring 的 `boolean isSingleton(String name) throws NoSuchBeanDefinitionException`。
    fn is_singleton(
        &self,
        key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// 查询 Bean 是否为 prototype（Transient）。
    ///
    /// 对应 Spring 的 `boolean isPrototype(String name) throws NoSuchBeanDefinitionException`。
    fn is_prototype(
        &self,
        key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取 Bean 类型名。
    ///
    /// 对应 Spring 的 `Class<?> getType(String name) throws NoSuchBeanDefinitionException`。
    fn get_type(
        &self,
        key: &ComponentKey,
    ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取 Bean 的别名。
    ///
    /// 对应 Spring 的 `String[] getAliases(String name)`。
    fn get_aliases(&self, key: &ComponentKey) -> Vec<ComponentKey>;

    /// 获取 Bean 供应器（延迟/可选）。
    ///
    /// 对应 Spring 的 `<T> ObjectProvider<T> getBeanProvider(Class<T> requiredType)`。
    /// 使用 TypeId 替代泛型以保持 dyn 兼容。
    fn get_bean_provider_by_type_id(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<
        Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
        Box<dyn std::error::Error + Send + Sync>,
    >;

    /// 按类型检查类型是否匹配。
    ///
    /// 对应 Spring 的 `boolean isTypeMatch(String name, Class<?> typeToMatch)`。
    fn is_type_match(&self, key: &ComponentKey, type_id: std::any::TypeId) -> bool;
}
