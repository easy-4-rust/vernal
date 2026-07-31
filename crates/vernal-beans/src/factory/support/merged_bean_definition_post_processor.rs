//! MergedBeanDefinitionPostProcessor — Spring 风格的合并 Bean 定义后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MergedBeanDefinitionPostProcessor`。
//!
//! 在 Spring 中，`MergedBeanDefinitionPostProcessor` 扩展了 `BeanPostProcessor`，
//! 在 Bean 定义合并完成后（`postProcessMergedBeanDefinition`）和 Bean 实例销毁前
//! （`resetBeanDefinition`）提供回调。
//!
//! Bean 定义合并是将子 BeanDefinition 与父 BeanDefinition 合并的过程，
//! 合并后的定义包含完整的配置信息。
//!
//! ## 设计说明
//!
//! 在 vernal 中，Bean 定义合并通过 `RootBeanDefinition` 的合并方法完成。
//! 此 trait 为后处理器提供合并后的通知回调。

use std::any::Any;
use std::sync::Arc;

/// 合并 Bean 定义后处理器接口。
///
/// 对应 Spring 的 `MergedBeanDefinitionPostProcessor`。
///
/// 在 Bean 定义合并后执行自定义处理逻辑。
/// 可用于注入元数据收集、依赖关系分析等。
pub trait MergedBeanDefinitionPostProcessor: Send + Sync {
    /// 合并 Bean 定义后处理。
    ///
    /// 对应 Spring 的 `postProcessMergedBeanDefinition(RootBeanDefinition, Class<?>, String)`。
    ///
    /// 在 Bean 定义合并完成后调用，可以在此方法中
    /// 收集注入元数据、验证定义完整性等。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `bean_type_name` — Bean 类型名
    ///
    /// # 返回
    /// 处理后的额外元数据（可选），或错误。
    fn post_process_merged_bean_definition(
        &self,
        bean_name: &str,
        bean_type_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;

    /// 重置 Bean 定义。
    ///
    /// 对应 Spring 的 `resetBeanDefinition(String)`。
    ///
    /// 当 Bean 定义被重新注册或销毁时调用，
    /// 用于清理之前收集的缓存元数据。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    fn reset_bean_definition(&self, _bean_name: &str) {
        // 默认空实现，子类按需覆盖
    }
}

/// 元数据收集型合并后处理器。
///
/// 收集已处理的 Bean 定义元数据，用于后续查询。
#[derive(Debug, Default)]
pub struct MetadataCollectingPostProcessor {
    processed_bean_types: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl MetadataCollectingPostProcessor {
    /// 创建元数据收集后处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取已处理的 Bean 类型名。
    pub fn get_processed_type(&self, bean_name: &str) -> Option<String> {
        self.processed_bean_types
            .lock()
            .unwrap()
            .get(bean_name)
            .cloned()
    }

    /// 获取已处理的 Bean 数量。
    pub fn processed_count(&self) -> usize {
        self.processed_bean_types.lock().unwrap().len()
    }
}

impl MergedBeanDefinitionPostProcessor for MetadataCollectingPostProcessor {
    fn post_process_merged_bean_definition(
        &self,
        bean_name: &str,
        bean_type_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        self.processed_bean_types
            .lock()
            .unwrap()
            .insert(bean_name.to_string(), bean_type_name.to_string());
        Ok(None)
    }

    fn reset_bean_definition(&self, bean_name: &str) {
        self.processed_bean_types.lock().unwrap().remove(bean_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_processor_records_bean_types() {
        let processor = MetadataCollectingPostProcessor::new();
        processor.post_process_merged_bean_definition("myBean", "com.example.MyService").unwrap();
        assert_eq!(processor.get_processed_type("myBean"), Some("com.example.MyService".to_string()));
        assert_eq!(processor.processed_count(), 1);
    }

    #[test]
    fn reset_bean_definition_removes_metadata() {
        let processor = MetadataCollectingPostProcessor::new();
        processor.post_process_merged_bean_definition("bean1", "Type1").unwrap();
        processor.post_process_merged_bean_definition("bean2", "Type2").unwrap();
        assert_eq!(processor.processed_count(), 2);

        processor.reset_bean_definition("bean1");
        assert_eq!(processor.processed_count(), 1);
        assert!(processor.get_processed_type("bean1").is_none());
        assert!(processor.get_processed_type("bean2").is_some());
    }

    #[test]
    fn get_processed_type_returns_none_for_unknown() {
        let processor = MetadataCollectingPostProcessor::new();
        assert!(processor.get_processed_type("nonexistent").is_none());
    }

    #[test]
    fn post_process_returns_none_metadata() {
        let processor = MetadataCollectingPostProcessor::new();
        let result = processor.post_process_merged_bean_definition("bean", "Type").unwrap();
        assert!(result.is_none());
    }
}
