//! InstantiationAwareBeanPostProcessor — Spring 风格的实例化感知后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.InstantiationAwareBeanPostProcessor`。
//!
//! 该接口扩展了 `BeanPostProcessor`，增加了实例化前后的回调。
//! 这是 Spring AOP、字段注入等机制的核心扩展点。

use std::any::Any;
use std::sync::Arc;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的实例化感知后处理器接口。
///
/// 对应 Spring 的 `InstantiationAwareBeanPostProcessor`。
///
/// 与 `BeanPostProcessor` 不同，此接口在 Bean **实例化之前**就可以拦截，
/// 用于短路 Bean 创建（例如返回缓存的代理对象）。
///
/// ## 方法优先级（按 Spring 7.0.8 语义）
///
/// 1. `post_process_before_instantiation` — 实例化**之前**调用
///    （如果返回非 None，后续实例化/属性注入/初始化全部跳过）
/// 2. （容器创建 Bean 实例 + 属性注入）
/// 3. `post_process_after_instantiation` — 实例化**之后**调用
///    （返回 false 阻止属性注入）
/// 4. `post_process_properties` — 属性注入阶段（处理 `@Autowired`）
/// 5. `BeanPostProcessor.post_process_before_initialization`
/// 6. `BeanPostProcessor.post_process_after_initialization`
///
/// ## 注意
///
/// Spring 7.0.8 中 `post_process_property_values` 已被移除（Spring 5.3），
/// 只保留 `post_process_properties`。
pub trait InstantiationAwareBeanPostProcessor: BeanPostProcessor {
    /// 在 Bean 实例化**之前**调用。
    ///
    /// 对应 Spring 的 `InstantiationAwareBeanPostProcessor.postProcessBeforeInstantiation(Class<?> beanClass, String beanName)`。
    ///
    /// 如果返回 `Some(proxy)`，容器会跳过后续的实例化、属性注入和初始化，
    /// 直接使用返回的代理对象。这是 Spring AOP 短路创建的核心机制。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(proxy))` — 短路创建，跳过后续流程
    /// - `Ok(None)` — 继续正常创建流程
    /// - `Err` — 阻止 Bean 创建
    fn post_process_before_instantiation(
        &self,
        _bean_type: &dyn std::any::Any,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// 在 Bean 实例化**之后**调用。
    ///
    /// 对应 Spring 的 `InstantiationAwareBeanPostProcessor.postProcessAfterInstantiation(Object bean, String beanName)`。
    ///
    /// 返回 `false` 时，容器会跳过后续的属性注入（populateBean）。
    /// 默认返回 `true`（继续属性注入）。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` — 继续属性注入
    /// - `Ok(false)` — 跳过属性注入
    /// - `Err` — 阻止 Bean 创建
    fn post_process_after_instantiation(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }

    /// 在属性注入阶段调用（处理 `@Autowired` 等注解）。
    ///
    /// 对应 Spring 的 `InstantiationAwareBeanPostProcessor.postProcessProperties(PropertyValues pvs, Object bean, String beanName)`。
    ///
    /// Spring 7.0.8 中只有此方法（`postProcessPropertyValues` 已移除）。
    /// 默认返回 `None`（不做修改）。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(modified_pvs))` — 修改后的属性值
    /// - `Ok(None)` — 使用原始属性值
    /// - `Err` — 阻止 Bean 创建
    fn post_process_properties(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<
        Option<Vec<(String, Arc<dyn Any + Send + Sync>)>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(None)
    }
}
