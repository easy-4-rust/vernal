//! FactoryBean — Spring 风格的工厂 Bean 接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.FactoryBean`。
//!
//! 工厂 Bean 是一种特殊的 Bean，它本身不作为最终产品暴露，
//! 而是作为其他 Bean 的工厂。容器在 `getBean("myFactory")` 时，
//! 会自动调用 `getObject()` 返回产品。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的工厂 Bean 接口。
///
/// 对应 Spring 的 `FactoryBean<T>`。
///
/// 工厂 Bean 是一种特殊的 Bean，容器在 `getBean` 时自动调用 `get_object`
/// 返回产品实例。
///
/// ## 使用场景
///
/// - 复杂对象的创建（连接池、客户端）
/// - 第三方库对象的适配
/// - 动态代理（AOP 代理工厂）
///
/// ## 前缀约定
///
/// 对应 Spring 的 `FACTORY_BEAN_PREFIX = "&"`。
/// - `getBean("myFactory")` → 调用 `myFactory.getObject()`
/// - `getBean("&myFactory")` → 返回 FactoryBean 本身
pub trait FactoryBean: Send + Sync + 'static {
    /// FactoryBean 前缀：`"&"`。
    const OBJECT_TYPE_ATTRIBUTE: &'static str = "factoryBeanObjectType";

    /// 获取产品实例。
    ///
    /// 对应 Spring 的 `T getObject() throws Exception`。
    ///
    /// 每次调用都返回一个新的产品实例（prototype）或同一个实例（singleton）。
    fn get_object(
        &self,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取产品类型。
    ///
    /// 对应 Spring 的 `Class<?> getObjectType()`。
    ///
    /// 返回产品实例的类型，用于按类型匹配。返回 `None` 表示类型未知。
    fn get_object_type(&self) -> Option<std::any::TypeId>;

    /// 是否是 singleton 产品。
    ///
    /// 对应 Spring 的 `default boolean isSingleton()`，默认返回 `true`。
    ///
    /// - `true` — 容器缓存 `getObject()` 的返回值
    /// - `false` — 每次 `getBean` 都调用 `getObject()` 创建新实例
    fn is_singleton(&self) -> bool {
        true
    }
}

/// SmartFactoryBean — Spring 风格的智能工厂 Bean 接口。
///
/// 对应 Java 类：`org.springframework.beans.factory.SmartFactoryBean`。
///
/// 扩展 `FactoryBean`，提供额外的控制能力：
/// - `is_eager_init` — 是否急切实例化（即使 lazy-init）
pub trait SmartFactoryBean: FactoryBean {
    /// 是否急切实例化。
    ///
    /// 对应 Spring 的 `boolean isEagerInit()`。
    ///
    /// 即使容器配置了 `lazy-init = true`，如果 `FactoryBean` 返回 `true`，
    /// 容器仍然会在启动时实例化该 Bean。
    fn is_eager_init(&self) -> bool;
}
