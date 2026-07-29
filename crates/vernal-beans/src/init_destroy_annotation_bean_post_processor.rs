//! InitDestroyAnnotationBeanPostProcessor — Spring 风格的 @PostConstruct/@PreDestroy 处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.InitDestroyAnnotationBeanPostProcessor`。
//!
//! 处理 `@PostConstruct` 和 `@PreDestroy` 注解。
//! 在 vernal-beans 中，初始化/销毁通过 `Component::inner_init` / `Component::shutdown` 管理。

use std::any::Any;
use std::sync::Arc;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的 @PostConstruct/@PreDestroy 注解后处理器。
///
/// 对应 Spring 的 `InitDestroyAnnotationBeanPostProcessor`。
///
/// 在 vernal 中，初始化/销毁通过 Component trait 的生命周期方法管理。
pub struct InitDestroyAnnotationBeanPostProcessor;

impl InitDestroyAnnotationBeanPostProcessor {
    /// 创建新的 InitDestroyAnnotationBeanPostProcessor。
    pub fn new() -> Self {
        Self
    }
}

impl Default for InitDestroyAnnotationBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanPostProcessor for InitDestroyAnnotationBeanPostProcessor {
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
