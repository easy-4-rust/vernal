//! SmartInstantiationAwareBeanPostProcessor — Spring 风格的智能实例化感知后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.SmartInstantiationAwareBeanPostProcessor`。
//!
//! 扩展 `InstantiationAwareBeanPostProcessor`，提供类型预测、构造器推断、
//! 循环引用处理等高级能力。

use std::any::Any;
use std::sync::Arc;

use crate::instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;

/// Spring 风格的智能实例化感知后处理器接口。
///
/// 对应 Spring 的 `SmartInstantiationAwareBeanPostProcessor`。
///
/// 此接口提供了以下高级能力：
///
/// - `predictBeanType` — 预测 Bean 类型（AOP 代理场景）
/// - `determineCandidateConstructors` — 推断候选构造器（构造器注入）
/// - `getEarlyBeanReference` — 提前获取 Bean 引用（循环引用解决）
pub trait SmartInstantiationAwareBeanPostProcessor: InstantiationAwareBeanPostProcessor {
    /// 预测 Bean 的最终类型。
    ///
    /// 对应 Spring 的 `SmartInstantiationAwareBeanPostProcessor.predictBeanType(Class<?> beanClass, String beanName)`。
    ///
    /// 用于在实例化前预测 Bean 的实际类型（例如 AOP 代理会返回代理类型）。
    /// 返回 `None` 表示不做预测。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(predicted_type))` — 预测的类型
    /// - `Ok(None)` — 不做预测，使用原始类型
    /// - `Err` — 类型预测失败
    fn predict_bean_type(
        &self,
        _bean_type: &dyn Any,
        _bean_name: &str,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(None)
    }

    /// 推断候选构造器。
    ///
    /// 对应 Spring 的 `SmartInstantiationAwareBeanPostProcessor.determineCandidateConstructors(Class<?> beanClass, String beanName)`。
    ///
    /// 用于构造器注入场景：当容器需要通过构造器创建 Bean 时，
    /// 调用此方法获取候选构造器列表。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(candidates))` — 候选构造器列表（Rust 中为空 vec 表示使用默认构造器）
    /// - `Ok(None)` — 不做推断，使用默认构造器
    /// - `Err` — 推断失败
    fn determine_candidate_constructors(
        &self,
        _bean_type: &dyn Any,
        _bean_name: &str,
    ) -> Result<Option<Vec<Box<dyn Any + Send + Sync>>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(None)
    }

    /// 提前获取 Bean 引用（用于解决循环依赖）。
    ///
    /// 对应 Spring 的 `SmartInstantiationAwareBeanPostProcessor.getEarlyBeanReference(Object bean, String beanName)`。
    ///
    /// 当 Bean 还在创建过程中，另一个 Bean 需要引用它时（循环依赖），
    /// 容器会调用此方法获取提前暴露的引用（通常是 AOP 代理的早期引用）。
    ///
    /// # 返回
    ///
    /// - `Ok(early_reference)` — 提前暴露的引用
    /// - `Err` — 获取失败
    fn get_early_bean_reference(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(bean)
    }
}
