//! BeanPostProcessor — Spring 风格的 Bean 后处理器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanPostProcessor`。
//!
//! 容器在 Bean 实例化后、属性注入完成后，对每个 Bean 调用 `BeanPostProcessor` 链。
//! 用户通过注册 `BeanPostProcessor` 实现来拦截 Bean 的初始化过程。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的 Bean 后处理器接口。
///
/// 对应 Spring 的 `BeanPostProcessor`。
///
/// 容器在每个 Bean 的 `afterPropertiesSet` 前后各调用一次：
/// - `post_process_before_initialization` — 在 `afterPropertiesSet` 之前
/// - `post_process_after_initialization` — 在 `afterPropertiesSet` 之后
///
/// ## 默认行为
///
/// 两个方法默认返回传入的 `bean`（不做任何修改）。
/// 这与 Spring 的 `BeanPostProcessor` 默认行为一致。
///
/// ## 使用场景
///
/// - AOP 代理包装
/// - 自定义注解处理（`@PostConstruct`）
/// - Bean 属性验证
/// - Bean 包装（如返回代理对象）
///
/// ## 与 vernal-aop 的关系
///
/// `vernal-aop` 的 `InterceptorChain` 可以通过实现 `BeanPostProcessor` 注入容器。
/// 这样每个 Bean 在初始化前后都会经过 AOP 拦截链。
pub trait BeanPostProcessor: Send + Sync + 'static {
    /// 在 Bean 的 `afterPropertiesSet` 之前调用。
    ///
    /// 对应 Spring 的 `BeanPostProcessor.postProcessBeforeInitialization(Object bean, String beanName)`。
    ///
    /// # 参数
    ///
    /// - `bean` — 已实例化但未完成初始化的 Bean（类型擦除）
    /// - `bean_name` — Bean 在容器中的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Some(new_bean))` — 返回替代 Bean（例如 AOP 代理）
    /// - `Ok(None)` — 返回原始 Bean
    /// - `Err` — 阻止 Bean 创建，抛出异常
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }

    /// 在 Bean 的 `afterPropertiesSet` 之后调用。
    ///
    /// 对应 Spring 的 `BeanPostProcessor.postProcessAfterInitialization(Object bean, String beanName)`。
    ///
    /// # 参数
    ///
    /// - `bean` — 已完成初始化的 Bean（类型擦除）
    /// - `_bean_name` — Bean 在容器中的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Some(new_bean))` — 返回替代 Bean（例如 AOP 代理）
    /// - `Ok(None)` — 返回原始 Bean
    /// - `Err` — 阻止 Bean 创建，抛出异常
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }
}
