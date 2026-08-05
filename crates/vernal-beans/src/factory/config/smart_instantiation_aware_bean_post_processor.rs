//! SmartInstantiationAwareBeanPostProcessor — 对应 Spring `org.springframework.beans.factory.config.SmartInstantiationAwareBeanPostProcessor`。
//!
//! 智能感知实例化的 BeanPostProcessor。

use std::any::Any;
use std::sync::Arc;

/// 智能感知实例化的 BeanPostProcessor。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.SmartInstantiationAwareBeanPostProcessor`。
///
/// 扩展了 `InstantiationAwareBeanPostProcessor`，增加了：
/// - 预测 Bean 类型
/// - 确定构造器
/// - 获取早期 Bean 引用
///
/// ## 使用场景
///
/// - AOP 代理创建（如 Spring AOP）
/// - 循环依赖解决
/// - 构造器选择
pub trait SmartInstantiationAwareBeanPostProcessor: Send + Sync {
    /// 预测 Bean 类型。
    ///
    /// 返回 Bean 的最终类型（可能是代理类型）。
    /// 默认返回 `None`（使用原始类型）。
    fn predict_bean_type(&self, _bean_class: &str, _bean_name: &str) -> Option<String> {
        None
    }

    /// 确定候选构造器。
    ///
    /// 返回候选构造器的参数类型列表。
    /// 默认返回 `None`（使用默认构造器）。
    fn determine_candidate_constructors(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Option<Vec<Vec<String>>> {
        None
    }

    /// 获取早期 Bean 引用。
    ///
    /// 用于解决循环依赖。
    /// 默认返回传入的 `early_reference`。
    fn get_early_bean_reference(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Arc<dyn Any + Send + Sync> {
        bean
    }

    /// 在 Bean 实例化之前调用。
    fn post_process_before_instantiation(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// 在 Bean 实例化之后调用。
    fn post_process_after_instantiation(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }

    /// 在 Bean 的 `afterPropertiesSet` 之前调用。
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }

    /// 在 Bean 的 `afterPropertiesSet` 之后调用。
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }
}

/// 简单的 SmartInstantiationAwareBeanPostProcessor 实现。
pub struct SimpleSmartInstantiationAwareBeanPostProcessor {
    /// 预测的 Bean 类型。
    predicted_type: Option<String>,
    /// 候选构造器。
    candidate_constructors: Option<Vec<Vec<String>>>,
}

impl SimpleSmartInstantiationAwareBeanPostProcessor {
    /// 创建新的 SimpleSmartInstantiationAwareBeanPostProcessor。
    pub fn new() -> Self {
        Self {
            predicted_type: None,
            candidate_constructors: None,
        }
    }

    /// 设置预测的 Bean 类型。
    pub fn set_predicted_type(&mut self, predicted_type: impl Into<String>) {
        self.predicted_type = Some(predicted_type.into());
    }

    /// 设置候选构造器。
    pub fn set_candidate_constructors(&mut self, constructors: Vec<Vec<String>>) {
        self.candidate_constructors = Some(constructors);
    }
}

impl Default for SimpleSmartInstantiationAwareBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartInstantiationAwareBeanPostProcessor for SimpleSmartInstantiationAwareBeanPostProcessor {
    fn predict_bean_type(&self, _bean_class: &str, _bean_name: &str) -> Option<String> {
        self.predicted_type.clone()
    }

    fn determine_candidate_constructors(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Option<Vec<Vec<String>>> {
        self.candidate_constructors.clone()
    }

    fn get_early_bean_reference(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Arc<dyn Any + Send + Sync> {
        bean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_instantiation_aware_bean_post_processor_default() {
        let processor = SimpleSmartInstantiationAwareBeanPostProcessor::new();

        // 默认返回 None
        assert!(processor.predict_bean_type("MyClass", "myBean").is_none());
        assert!(
            processor
                .determine_candidate_constructors("MyClass", "myBean")
                .is_none()
        );
    }

    #[test]
    fn test_smart_instantiation_aware_bean_post_processor_predict_type() {
        let mut processor = SimpleSmartInstantiationAwareBeanPostProcessor::new();
        processor.set_predicted_type("com.example.Proxy");

        assert_eq!(
            processor.predict_bean_type("MyClass", "myBean"),
            Some("com.example.Proxy".to_string())
        );
    }

    #[test]
    fn test_smart_instantiation_aware_bean_post_processor_early_reference() {
        let processor = SimpleSmartInstantiationAwareBeanPostProcessor::new();
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test"));
        let early_ref = processor.get_early_bean_reference(bean.clone(), "myBean");

        assert!(Arc::ptr_eq(&bean, &early_ref));
    }
}
