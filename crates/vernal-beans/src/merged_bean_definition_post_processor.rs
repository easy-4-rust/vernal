//! MergedBeanDefinitionPostProcessor — 合并 Bean 定义后处理器。
use crate::bean_post_processor::BeanPostProcessor;

/// 合并 Bean 定义后处理器 trait。
pub trait MergedBeanDefinitionPostProcessor: BeanPostProcessor {
    fn post_process_merged_bean_definition(&self, bean_type: &str, bean_name: &str);
}
