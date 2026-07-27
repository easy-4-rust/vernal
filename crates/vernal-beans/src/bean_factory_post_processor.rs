//! BeanFactoryPostProcessor — Spring 风格的 Bean 工厂后处理器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanFactoryPostProcessor`。
//!
//! 在所有 Bean 定义加载完成后、Bean 实例化之前调用。
//! 典型用途：修改 Bean 定义（如属性占位符替换）。

use crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

/// Spring 风格的 Bean 工厂后处理器接口。
///
/// 对应 Spring 的 `BeanFactoryPostProcessor`。
///
/// 在所有 Bean 定义加载完成后、Bean 实例化之前调用。
/// 典型用途：
/// - 属性占位符替换（`PropertyPlaceholderConfigurer`）
/// - 自定义 Bean 定义修改
/// - 属性源注册
///
/// ## 执行顺序
///
/// 1. `BeanDefinitionRegistryPostProcessor.postProcessBeanDefinitionRegistry`
/// 2. **`BeanFactoryPostProcessor.postProcessBeanFactory`** ← 此接口
/// 3. Bean 实例化
pub trait BeanFactoryPostProcessor: Send + Sync + 'static {
    /// 在所有 Bean 定义加载完成后调用。
    ///
    /// 对应 Spring 的 `void postProcessBeanFactory(ConfigurableListableBeanFactory beanFactory)`。
    fn post_process_bean_factory(
        &self,
        bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
