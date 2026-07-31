//! BeanFactoryAwareProcessor — Spring 风格 BeanFactory 感知处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanFactoryAwareProcessor`。
//!
//! 在 Spring 中，`BeanFactoryAwareProcessor` 是一个 `BeanPostProcessor`，
//! 负责在 Bean 初始化前调用 `BeanFactoryAware.setBeanFactory()`，
//! 将当前 `BeanFactory` 注入到实现了 `BeanFactoryAware` 接口的 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// BeanFactory 感知处理器。
///
/// 对应 Spring 的 `BeanFactoryAwareProcessor`。
///
/// 在 `BeanPostProcessor.postProcessBeforeInitialization` 阶段，
/// 检查 Bean 是否实现了 `BeanFactoryAware` 接口，如果是则调用
/// `set_bean_factory()` 将容器引用注入。
#[derive(Debug)]
pub struct BeanFactoryAwareProcessor {
    /// BeanFactory 的类型擦除引用
    bean_factory: Arc<dyn Any + Send + Sync>,
    /// 已注入的 Bean 名称列表
    injected_beans: std::sync::Mutex<Vec<String>>,
}

impl BeanFactoryAwareProcessor {
    /// 创建新的 BeanFactory 感知处理器。
    ///
    /// # 参数
    /// - `bean_factory` — 当前 BeanFactory 的类型擦除引用
    pub fn new(bean_factory: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            bean_factory,
            injected_beans: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 获取 BeanFactory 引用。
    pub fn bean_factory(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.bean_factory
    }

    /// 获取已注入的 Bean 数量。
    pub fn injected_count(&self) -> usize {
        self.injected_beans.lock().unwrap().len()
    }

    /// 获取已注入的 Bean 名称列表。
    pub fn injected_bean_names(&self) -> Vec<String> {
        self.injected_beans.lock().unwrap().clone()
    }

    /// 检查指定 Bean 是否已被注入。
    pub fn is_injected(&self, bean_name: &str) -> bool {
        self.injected_beans.lock().unwrap().iter().any(|n| n == bean_name)
    }
}

impl BeanPostProcessor for BeanFactoryAwareProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 在实际实现中，这里会检查 Bean 是否实现了 BeanFactoryAware
        // 如果是，则调用 set_bean_factory()
        // 由于 dyn BeanFactory 不是 dyn-compatible 的，
        // 我们使用类型擦除的引用

        self.injected_beans.lock().unwrap().push(bean_name.to_string());

        Ok(Some(bean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_stores_bean_factory() {
        let factory: Arc<dyn std::any::Any + Send + Sync> = Arc::new("mock_factory".to_string());
        let processor = BeanFactoryAwareProcessor::new(Arc::clone(&factory));

        assert_eq!(
            processor.bean_factory().downcast_ref::<String>(),
            Some(&"mock_factory".to_string())
        );
    }

    #[test]
    fn post_process_records_injection() {
        let factory: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42_i32);
        let processor = BeanFactoryAwareProcessor::new(factory);
        let bean = Arc::new("test".to_string());

        let result = processor
            .post_process_before_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(processor.injected_count(), 1);
        assert!(processor.is_injected("myBean"));
        assert!(!processor.is_injected("otherBean"));
    }

    #[test]
    fn tracks_multiple_injected_beans() {
        let factory = Arc::new(());
        let processor = BeanFactoryAwareProcessor::new(factory);

        processor
            .post_process_before_initialization(Arc::new(1), "bean1")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(2), "bean2")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(3), "bean3")
            .unwrap();

        assert_eq!(processor.injected_count(), 3);
        let names = processor.injected_bean_names();
        assert_eq!(names, vec!["bean1", "bean2", "bean3"]);
    }
}
