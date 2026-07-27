//! BeanNameAware — Spring 风格的 Bean 名称感知接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanNameAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_bean_name`，
//! 将 Bean 在容器中的名称注入 Bean。

use crate::aware::Aware;

/// Spring 风格的 Bean 名称感知接口。
///
/// 对应 Spring 的 `BeanNameAware.setBeanName(String name)`。
///
/// 容器在实例化 Bean 后、`InitializingBean.afterPropertiesSet()` 之前，
/// 调用此方法将 Bean 名称注入。
///
/// # 优先级
///
/// Bean 名称感知回调在以下顺序中执行（对标 Spring 生命周期）：
/// 1. `BeanNameAware.setBeanName`
/// 2. `BeanFactoryAware.setBeanFactory`
/// 3. `BeanPostProcessor.postProcessBeforeInitialization`
/// 4. `InitializingBean.afterPropertiesSet`
/// 5. 自定义 `init-method`
/// 6. `BeanPostProcessor.postProcessAfterInitialization`
pub trait BeanNameAware: Aware {
    /// 将 Bean 名称注入 Bean。
    ///
    /// 对应 Spring 的 `BeanNameAware.setBeanName(String name)`。
    fn set_bean_name(&mut self, name: &str);
}
