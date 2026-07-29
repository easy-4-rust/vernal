//! AutowiredAnnotationBeanPostProcessor — Spring 风格的 @Autowired 后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor`。
//!
//! 处理 `@Autowired`、`@Value` 注解驱动的自动装配。
//! 在 vernal-beans 中，自动装配通过 Resolver 在 ComponentDefinition 工厂闭包中完成，
//! 此 post-processor 提供与 Spring 注解后处理器的兼容层。

use std::any::Any;
use std::sync::Arc;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的 Autowired 注解后处理器。
///
/// 对应 Spring 的 `AutowiredAnnotationBeanPostProcessor`。
///
/// 处理字段和方法的 `@Autowired` / `@Value` 注解。
/// 在 vernal-beans 中解析依赖于 Resolver 提供的依赖图。
pub struct AutowiredAnnotationBeanPostProcessor;

impl AutowiredAnnotationBeanPostProcessor {
    /// 创建新的 AutowiredAnnotationBeanPostProcessor。
    pub fn new() -> Self {
        Self
    }
}

impl Default for AutowiredAnnotationBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanPostProcessor for AutowiredAnnotationBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 在 vernal 中，自动装配在 ComponentDefinition 工厂闭包中已完成
        Ok(None)
    }

    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}
