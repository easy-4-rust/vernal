//! InitializingBean — Spring 风格的初始化回调接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.InitializingBean`。
//!
//! Bean 实现此接口后，容器在所有属性注入完成后回调 `after_properties_set`。

use crate::factory::aware::Aware;

/// Spring 风格的初始化回调接口。
///
/// 对应 Spring 的 `InitializingBean.afterPropertiesSet()`。
///
/// 在所有属性注入完成后、`BeanPostProcessor.postProcessBeforeInitialization` 之前，
/// 容器调用此方法进行同步初始化。
///
/// ## 执行顺序（对标 Spring 生命周期）
///
/// 1. 构造器注入
/// 2. `BeanNameAware.setBeanName`
/// 3. `BeanFactoryAware.setBeanFactory`
/// 4. `BeanPostProcessor.postProcessBeforeInitialization`
/// 5. **`InitializingBean.afterPropertiesSet`** ← 此接口
/// 6. 自定义 `init-method`
/// 7. `BeanPostProcessor.postProcessAfterInitialization`
///
/// ## 与 vernal 现有体系的关系
///
/// - `vernal-beans::component_contract::Component::inner_init` 对应此语义
/// - 保留此 trait 是为了与 Spring 用户的编程习惯对齐
pub trait InitializingBean: Aware {
    /// 所有属性注入完成后调用。
    ///
    /// 对应 Spring 的 `InitializingBean.afterPropertiesSet()`。
    ///
    /// # 错误
    ///
    /// 返回 `Err` 时容器会抛出 `BeanCreationException`。
    fn after_properties_set(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
