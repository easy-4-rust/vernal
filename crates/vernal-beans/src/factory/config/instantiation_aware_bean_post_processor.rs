//! InstantiationAwareBeanPostProcessor — 对应 Spring `org.springframework.beans.factory.config.InstantiationAwareBeanPostProcessor`。
//!
//! 感知实例化的 BeanPostProcessor。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// 感知实例化的 BeanPostProcessor。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.InstantiationAwareBeanPostProcessor`。
///
/// 扩展了 `BeanPostProcessor`，增加了在 Bean 实例化前后调用的方法。
///
/// ## 主要方法
///
/// - `post_process_before_instantiation` — 在 Bean 实例化之前调用
/// - `post_process_after_instantiation` — 在 Bean 实例化之后调用
/// - `post_process_property_values` — 在属性注入之前调用
///
/// ## 使用场景
///
/// - AOP 代理创建
/// - 属性注入拦截
/// - Bean 实例化控制
pub trait InstantiationAwareBeanPostProcessor: Send + Sync {
    /// 在 Bean 实例化之前调用。
    ///
    /// 返回 `Some(proxy)` 时跳过正常实例化流程。
    /// 默认返回 `None`（不跳过实例化）。
    fn post_process_before_instantiation(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// 在 Bean 实例化之后调用。
    ///
    /// 返回 `false` 时跳过属性注入。
    /// 默认返回 `true`（继续属性注入）。
    fn post_process_after_instantiation(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }

    /// 在属性注入之前调用。
    ///
    /// 返回修改后的属性值，或 `None` 表示不修改。
    /// 默认返回 `None`（不修改属性）。
    fn post_process_property_values(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
        properties: HashMap<String, Arc<dyn Any + Send + Sync>>,
    ) -> Result<
        Option<HashMap<String, Arc<dyn Any + Send + Sync>>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(Some(properties))
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

/// 简单的 InstantiationAwareBeanPostProcessor 实现。
pub struct SimpleInstantiationAwareBeanPostProcessor {
    /// 是否跳过实例化。
    skip_instantiation: bool,
    /// 是否继续属性注入。
    continue_injection: bool,
}

impl SimpleInstantiationAwareBeanPostProcessor {
    /// 创建新的 SimpleInstantiationAwareBeanPostProcessor。
    pub fn new() -> Self {
        Self {
            skip_instantiation: false,
            continue_injection: true,
        }
    }

    /// 设置是否跳过实例化。
    pub fn set_skip_instantiation(&mut self, skip: bool) {
        self.skip_instantiation = skip;
    }

    /// 设置是否继续属性注入。
    pub fn set_continue_injection(&mut self, continue_injection: bool) {
        self.continue_injection = continue_injection;
    }
}

impl Default for SimpleInstantiationAwareBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstantiationAwareBeanPostProcessor for SimpleInstantiationAwareBeanPostProcessor {
    fn post_process_before_instantiation(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        if self.skip_instantiation {
            Ok(Some(Arc::new(String::from("proxy"))))
        } else {
            Ok(None)
        }
    }

    fn post_process_after_instantiation(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.continue_injection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instantiation_aware_bean_post_processor_default() {
        let processor = SimpleInstantiationAwareBeanPostProcessor::new();

        // 默认不跳过实例化
        let result = processor.post_process_before_instantiation("MyClass", "myBean").unwrap();
        assert!(result.is_none());

        // 默认继续属性注入
        let bean = Arc::new(String::from("test"));
        let result = processor.post_process_after_instantiation(bean.as_ref(), "myBean").unwrap();
        assert!(result);
    }

    #[test]
    fn test_instantiation_aware_bean_post_processor_skip() {
        let mut processor = SimpleInstantiationAwareBeanPostProcessor::new();
        processor.set_skip_instantiation(true);

        let result = processor.post_process_before_instantiation("MyClass", "myBean").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_instantiation_aware_bean_post_processor_properties() {
        let processor = SimpleInstantiationAwareBeanPostProcessor::new();
        let bean = Arc::new(String::from("test"));
        let mut properties = HashMap::new();
        properties.insert("key".to_string(), Arc::new(42i32) as Arc<dyn Any + Send + Sync>);

        let result = processor.post_process_property_values(bean.as_ref(), "myBean", properties).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_default_trait_creates_default() {
        let processor = SimpleInstantiationAwareBeanPostProcessor::default();
        let result = processor.post_process_before_instantiation("MyClass", "myBean").unwrap();
        assert!(result.is_none()); // default doesn't skip
    }

    #[test]
    fn test_set_continue_injection_false() {
        let mut processor = SimpleInstantiationAwareBeanPostProcessor::new();
        processor.set_continue_injection(false);
        let bean = Arc::new(String::from("test"));
        let result = processor.post_process_after_instantiation(bean.as_ref(), "myBean").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_set_skip_instantiation_and_continue_injection() {
        let mut processor = SimpleInstantiationAwareBeanPostProcessor::new();
        processor.set_skip_instantiation(true);
        processor.set_continue_injection(false);

        // Skip instantiation returns proxy
        let result = processor.post_process_before_instantiation("MyClass", "myBean").unwrap();
        assert!(result.is_some());

        // Continue injection is false
        let bean = Arc::new(String::from("test"));
        let result = processor.post_process_after_instantiation(bean.as_ref(), "myBean").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_post_process_before_initialization_default() {
        struct DefaultProcessor;
        impl InstantiationAwareBeanPostProcessor for DefaultProcessor {}

        let processor = DefaultProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test"));
        let result = processor.post_process_before_initialization(bean.clone(), "myBean").unwrap();
        assert!(result.is_some());
        // Default returns the same bean
        assert!(Arc::ptr_eq(&result.unwrap(), &bean));
    }

    #[test]
    fn test_post_process_after_initialization_default() {
        struct DefaultProcessor;
        impl InstantiationAwareBeanPostProcessor for DefaultProcessor {}

        let processor = DefaultProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test"));
        let result = processor.post_process_after_initialization(bean.clone(), "myBean").unwrap();
        assert!(result.is_some());
        assert!(Arc::ptr_eq(&result.unwrap(), &bean));
    }

    #[test]
    fn test_post_process_property_values_default() {
        struct DefaultProcessor;
        impl InstantiationAwareBeanPostProcessor for DefaultProcessor {}

        let processor = DefaultProcessor;
        let bean = Arc::new(String::from("test"));
        let mut properties = HashMap::new();
        properties.insert("key".to_string(), Arc::new(42i32) as Arc<dyn Any + Send + Sync>);

        let result = processor.post_process_property_values(bean.as_ref(), "myBean", properties).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_post_process_before_instantiation_default() {
        struct DefaultProcessor;
        impl InstantiationAwareBeanPostProcessor for DefaultProcessor {}

        let processor = DefaultProcessor;
        let result = processor.post_process_before_instantiation("MyClass", "myBean").unwrap();
        assert!(result.is_none()); // default returns None
    }

    #[test]
    fn test_post_process_after_instantiation_default() {
        struct DefaultProcessor;
        impl InstantiationAwareBeanPostProcessor for DefaultProcessor {}

        let processor = DefaultProcessor;
        let bean = Arc::new(String::from("test"));
        let result = processor.post_process_after_instantiation(bean.as_ref(), "myBean").unwrap();
        assert!(result); // default returns true
    }

    #[test]
    fn test_properties_with_empty_map() {
        let processor = SimpleInstantiationAwareBeanPostProcessor::new();
        let bean = Arc::new(String::from("test"));
        let properties = HashMap::new();
        let result = processor.post_process_property_values(bean.as_ref(), "myBean", properties).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_properties_with_multiple_entries() {
        let processor = SimpleInstantiationAwareBeanPostProcessor::new();
        let bean = Arc::new(String::from("test"));
        let mut properties = HashMap::new();
        properties.insert("key1".to_string(), Arc::new(1i32) as Arc<dyn Any + Send + Sync>);
        properties.insert("key2".to_string(), Arc::new("value".to_string()) as Arc<dyn Any + Send + Sync>);
        properties.insert("key3".to_string(), Arc::new(true) as Arc<dyn Any + Send + Sync>);

        let result = processor.post_process_property_values(bean.as_ref(), "myBean", properties).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 3);
    }
}
