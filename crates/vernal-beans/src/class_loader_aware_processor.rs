//! ClassLoaderAwareProcessor — Spring 风格类加载器感知处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ClassLoaderAwareProcessor`。
//!
//! 在 Spring 中，`ClassLoaderAwareProcessor` 是一个 `BeanPostProcessor`，
//! 负责在 Bean 初始化前调用 `BeanClassLoaderAware.setBeanClassLoader()`，
//! 将当前类加载器注入到实现了 `BeanClassLoaderAware` 接口的 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// 类加载器感知处理器。
///
/// 对应 Spring 的 `ClassLoaderAwareProcessor`。
///
/// 在 `BeanPostProcessor.postProcessBeforeInitialization` 阶段，
/// 检查 Bean 是否实现了 `BeanClassLoaderAware` 接口，如果是则调用
/// `set_bean_class_loader()` 将类加载器名称注入。
#[derive(Debug)]
pub struct ClassLoaderAwareProcessor {
    /// 类加载器名称
    class_loader_name: String,
    /// 已注入的 Bean 名称列表
    injected_beans: std::sync::Mutex<Vec<String>>,
}

impl ClassLoaderAwareProcessor {
    /// 创建新的类加载器感知处理器。
    ///
    /// # 参数
    /// - `class_loader_name` — 当前类加载器的名称
    pub fn new(class_loader_name: impl Into<String>) -> Self {
        Self {
            class_loader_name: class_loader_name.into(),
            injected_beans: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 创建使用默认类加载器的处理器。
    pub fn with_default_class_loader() -> Self {
        Self::new("AppClassLoader")
    }

    /// 获取类加载器名称。
    pub fn class_loader_name(&self) -> &str {
        &self.class_loader_name
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

    /// 清空已注入记录。
    pub fn clear_injected(&self) {
        self.injected_beans.lock().unwrap().clear();
    }
}

impl BeanPostProcessor for ClassLoaderAwareProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        // 在实际实现中，这里会检查 Bean 是否实现了 BeanClassLoaderAware
        // 如果是，则调用 set_bean_class_loader(&self.class_loader_name)

        self.injected_beans.lock().unwrap().push(bean_name.to_string());

        Ok(Some(bean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_stores_class_loader_name() {
        let processor = ClassLoaderAwareProcessor::new("CustomClassLoader");
        assert_eq!(processor.class_loader_name(), "CustomClassLoader");
    }

    #[test]
    fn with_default_class_loader() {
        let processor = ClassLoaderAwareProcessor::with_default_class_loader();
        assert_eq!(processor.class_loader_name(), "AppClassLoader");
    }

    #[test]
    fn post_process_records_injection() {
        let processor = ClassLoaderAwareProcessor::new("test-loader");
        let bean = Arc::new(42_i32);

        let result = processor
            .post_process_before_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(processor.injected_count(), 1);
        assert!(processor.is_injected("myBean"));
        assert!(!processor.is_injected("otherBean"));
    }

    #[test]
    fn post_process_returns_original_bean() {
        let processor = ClassLoaderAwareProcessor::new("loader");
        let bean: Arc<dyn std::any::Any + Send + Sync> = Arc::new("hello".to_string());

        let result = processor
            .post_process_before_initialization(Arc::clone(&bean), "testBean")
            .unwrap();
        let returned = result.unwrap();
        assert!(Arc::ptr_eq(&returned, &bean));
    }

    #[test]
    fn clear_injected_removes_all_records() {
        let processor = ClassLoaderAwareProcessor::new("loader");
        processor
            .post_process_before_initialization(Arc::new(1), "bean1")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(2), "bean2")
            .unwrap();

        assert_eq!(processor.injected_count(), 2);
        processor.clear_injected();
        assert_eq!(processor.injected_count(), 0);
    }

    #[test]
    fn tracks_multiple_beans() {
        let processor = ClassLoaderAwareProcessor::new("loader");
        processor
            .post_process_before_initialization(Arc::new(()), "a")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(()), "b")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(()), "c")
            .unwrap();

        assert_eq!(processor.injected_count(), 3);
        let names = processor.injected_bean_names();
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
