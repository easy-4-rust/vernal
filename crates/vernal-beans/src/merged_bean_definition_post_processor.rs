//! MergedBeanDefinitionPostProcessor — Spring 风格的合并 Bean 定义后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MergedBeanDefinitionPostProcessor`。
//!
//! 在 Bean 定义合并完成后调用，用于处理合并后的 Bean 元数据。

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的合并 Bean 定义后处理器接口。
///
/// 对应 Spring 的 `MergedBeanDefinitionPostProcessor`。
///
/// 在 `AbstractBeanFactory.doCreateBean` 中，当 Bean 定义合并完成后，
/// 容器对所有 `MergedBeanDefinitionPostProcessor` 调用 `post_process_merged_beandefinition`。
///
/// 典型用途：`AutowiredAnnotationBeanPostProcessor` 在此方法中缓存
/// `@Autowired` 注解的元数据。
pub trait MergedBeanDefinitionPostProcessor: BeanPostProcessor {
    /// 合并 Bean 定义完成后调用。
    ///
    /// 对应 Spring 的 `MergedBeanDefinitionPostProcessor.postProcessMergedBeanDefinition(RootBeanDefinition beanDefinition, Class<?> beanType, String beanName)`。
    fn post_process_merged_beandefinition(&self, bean_type_name: &str, bean_name: &str);

    /// 重置 Bean 定义（Bean 定义被覆盖时调用）。
    ///
    /// 对应 Spring 的 `MergedBeanDefinitionPostProcessor.resetBeanDefinition(String beanName)`。
    fn reset_beandefinition(&self, _bean_name: &str) {
        // 默认空实现
    }
}
