//! BeanDefinitionRegistryPostProcessor — Spring 风格的 Bean 定义注册表后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor`。
//!
//! 扩展 `BeanFactoryPostProcessor`，在所有 Bean 定义加载完成后
//! 可以动态注册新的 Bean 定义。

use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_factory_post_processor::BeanFactoryPostProcessor;
use crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

/// Spring 风格的 Bean 定义注册表后处理器接口。
///
/// 对应 Spring 的 `BeanDefinitionRegistryPostProcessor`。
///
/// 扩展 `BeanFactoryPostProcessor`，在所有 Bean 定义加载完成后
/// 可以动态注册新的 Bean 定义。这是 `@Configuration` / `@Bean`
/// 处理的核心扩展点。
///
/// ## 执行顺序
///
/// 1. **`postProcessBeanDefinitionRegistry`** ← 此方法（注册新 Bean 定义）
/// 2. `postProcessBeanFactory`（修改 Bean 定义）
/// 3. Bean 实例化
pub trait BeanDefinitionRegistryPostProcessor: BeanFactoryPostProcessor {
    /// 在所有 Bean 定义加载完成后调用，可以注册新的 Bean 定义。
    fn post_process_bean_definition_registry(
        &self,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 默认实现：委托给 `post_process_bean_factory`。
    fn post_process_bean_factory(
        &self,
        _bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 默认空实现，不要求子类覆盖
        Ok(())
    }
}
