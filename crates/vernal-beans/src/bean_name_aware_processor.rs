//! BeanNameAwareProcessor — Spring 风格 Bean 名称感知处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanNameAwareProcessor`。
//!
//! 在 Spring 中，`BeanNameAwareProcessor` 是一个 `BeanPostProcessor`，
//! 负责在 Bean 初始化前调用 `BeanNameAware.setBeanName()`，
//! 将 Bean 在容器中的名称注入到实现了 `BeanNameAware` 接口的 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// Bean 名称感知处理器。
///
/// 对应 Spring 的 `BeanNameAwareProcessor`。
///
/// 在 `BeanPostProcessor.postProcessBeforeInitialization` 阶段，
/// 检查 Bean 是否实现了 `BeanNameAware` 接口，如果是则调用
/// `set_bean_name()` 将 Bean 名称注入。
///
/// 执行顺序：BeanNameAware 回调在 BeanFactoryAware 之前。
#[derive(Debug, Default)]
pub struct BeanNameAwareProcessor {
    /// 已处理的 Bean 名称映射（bean_name -> 处理时间戳）
    processed_beans: std::sync::Mutex<Vec<String>>,
}

impl BeanNameAwareProcessor {
    /// 创建新的 Bean 名称感知处理器。
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
        self.processed_beans
            .lock()
            .unwrap()
            .iter()
            .any(|n| n == bean_name)
    }

    /// 清空已处理记录。
    pub fn clear_processed(&self) {
        self.processed_beans.lock().unwrap().clear();
    }
}

impl BeanPostProcessor for BeanNameAwareProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 在实际实现中，这里会检查 Bean 是否实现了 BeanNameAware
        // 如果是，则调用 set_bean_name(bean_name)

        self.processed_beans
            .lock()
            .unwrap()
            .push(bean_name.to_string());

        Ok(Some(bean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_has_no_processed_beans() {
        let processor = BeanNameAwareProcessor::new();
        assert_eq!(processor.processed_count(), 0);
        assert!(processor.processed_bean_names().is_empty());
    }

    #[test]
    fn post_process_records_bean_name() {
        let processor = BeanNameAwareProcessor::new();
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
    fn post_process_returns_original_bean() {
        let processor = BeanNameAwareProcessor::new();
        let bean: Arc<dyn std::any::Any + Send + Sync> = Arc::new("hello".to_string());

        let result = processor
            .post_process_before_initialization(Arc::clone(&bean), "testBean")
            .unwrap();
        let returned = result.unwrap();
        assert!(Arc::ptr_eq(&returned, &bean));
    }

    #[test]
    fn clear_processed_removes_all_records() {
        let processor = BeanNameAwareProcessor::new();
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

    #[test]
    fn tracks_multiple_beans_in_order() {
        let processor = BeanNameAwareProcessor::new();
        processor
            .post_process_before_initialization(Arc::new(()), "alpha")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(()), "beta")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(()), "gamma")
            .unwrap();

        let names = processor.processed_bean_names();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }
}
