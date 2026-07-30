//! SmartInstantiationAwareBeanPostProcessor — 智能实例化感知后处理器。
use crate::instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;

/// 智能实例化感知后处理器 trait。
pub trait SmartInstantiationAwareBeanPostProcessor: InstantiationAwareBeanPostProcessor {
    fn predict_bean_type(&self, bean_class: &str) -> Option<String> {
        Some(bean_class.to_string())
    }
    fn determine_candidate_constructors(&self, bean_class: &str) -> Vec<String> {
        vec!["new".to_string()]
    }
}
