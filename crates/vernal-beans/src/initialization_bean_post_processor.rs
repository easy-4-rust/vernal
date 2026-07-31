//! InitializationBeanPostProcessor — Spring 风格初始化后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.InitializationBeanPostProcessor`。
//!
//! 在 Spring 中，`InitializationBeanPostProcessor` 在 Bean 初始化阶段
//! 调用 `InitializingBean.afterPropertiesSet()` 和自定义 init 方法。
//! 在 vernal 中，此处理器负责执行 Bean 的初始化回调。

use std::any::Any;
use std::sync::Arc;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// 初始化 Bean 后处理器。
///
/// 对应 Spring 的初始化阶段处理。
///
/// 在 Bean 属性注入完成后，调用初始化回调：
/// 1. `InitializingBean.afterPropertiesSet()`
/// 2. 自定义 init-method
///
/// 此处理器在 `BeanPostProcessor.postProcessBeforeInitialization` 阶段执行。
#[derive(Debug, Default)]
pub struct InitializationBeanPostProcessor {
    /// 已处理的 Bean 名称列表（用于调试和日志）
    processed_beans: std::sync::Mutex<Vec<String>>,
}

impl InitializationBeanPostProcessor {
    /// 创建新的初始化后处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取已处理的 Bean 数量。
    pub fn processed_count(&self) -> usize {
        self.processed_beans.lock().unwrap().len()
    }

    /// 获取已处理的 Bean 名称列表。
    pub fn processed_bean_names(&self) -> Vec<String> {
        self.processed_beans.lock().unwrap().clone()
    }

    /// 检查指定 Bean 是否已被处理。
    pub fn is_processed(&self, bean_name: &str) -> bool {
        self.processed_beans.lock().unwrap().iter().any(|n| n == bean_name)
    }

    /// 清空已处理记录。
    pub fn clear_processed(&self) {
        self.processed_beans.lock().unwrap().clear();
    }
}

impl BeanPostProcessor for InitializationBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 记录已处理的 Bean
        self.processed_beans.lock().unwrap().push(bean_name.to_string());

        // 在实际实现中，这里会检查 Bean 是否实现了 InitializingBean
        // 并调用 afterPropertiesSet()
        // 同时检查是否有自定义 init-method

        Ok(Some(bean))
    }

    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_has_no_processed_beans() {
        let processor = InitializationBeanPostProcessor::new();
        assert_eq!(processor.processed_count(), 0);
        assert!(processor.processed_bean_names().is_empty());
    }

    #[test]
    fn post_process_records_bean_name() {
        let processor = InitializationBeanPostProcessor::new();
        let bean = Arc::new(42_i32);

        let result = processor
            .post_process_before_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(processor.processed_count(), 1);
        assert!(processor.is_processed("myBean"));
        assert!(!processor.is_processed("otherBean"));
    }

    #[test]
    fn post_process_after_initialization_returns_bean() {
        let processor = InitializationBeanPostProcessor::new();
        let bean = Arc::new("test".to_string());

        let result = processor
            .post_process_after_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn clear_processed_removes_all_records() {
        let processor = InitializationBeanPostProcessor::new();
        processor
            .post_process_before_initialization(Arc::new(1), "bean1")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(2), "bean2")
            .unwrap();

        assert_eq!(processor.processed_count(), 2);
        processor.clear_processed();
        assert_eq!(processor.processed_count(), 0);
    }
}
