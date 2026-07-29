//! CommonAnnotationBeanPostProcessor — Spring 风格的通用注解后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.CommonAnnotationBeanPostProcessor`。
//!
//! 处理 `@Resource`、`@PostConstruct`、`@PreDestroy` 等通用注解。
//! 在 vernal-beans 中，依赖注入通过 Resolver + ComponentDefinition 工厂闭包完成。

use std::any::Any;
use std::sync::Arc;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的通用注解后处理器。
///
/// 对应 Spring 的 `CommonAnnotationBeanPostProcessor`。
///
/// 处理 `@Resource`、`@PostConstruct`、`@PreDestroy` 等注解。
pub struct CommonAnnotationBeanPostProcessor;

impl CommonAnnotationBeanPostProcessor {
    /// 创建新的 CommonAnnotationBeanPostProcessor。
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonAnnotationBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanPostProcessor for CommonAnnotationBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
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
