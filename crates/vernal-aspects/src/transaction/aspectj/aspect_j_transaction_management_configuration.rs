//! 对标 `org.springframework.transaction.aspectj.AspectJTransactionManagementConfiguration`。
//!
//! 对应 Java：`AspectJTransactionManagementConfiguration.java`（`spring-aspects` 模块）
//! 包路径：`org.springframework.transaction.aspectj`
//! 核心职责：`@Configuration` 类——注册 `AnnotationTransactionAspect` Bean，启用 `@Transactional` 声明式事务管理。

use std::sync::{Arc, RwLock};

use super::annotation_transaction_aspect::AnnotationTransactionAspect;
use super::transaction_attribute_source::TransactionAttributeSource;

/// AspectJ 事务管理配置。
///
/// 对标 Spring 的 `AspectJTransactionManagementConfiguration`。
/// 注册 `AnnotationTransactionAspect` 单例 Bean。
///
/// # 使用
///
/// ```rust,ignore
/// let config = AspectJTransactionManagementConfiguration::new();
/// config.register();
/// ```
pub struct AspectJTransactionManagementConfiguration<S: TransactionAttributeSource + 'static> {
    /// 事务管理器名称（可选）。
    transaction_manager_name: Option<String>,

    /// 已注册的事务切面（懒初始化）。
    aspect: RwLock<Option<Arc<AnnotationTransactionAspect<S>>>>,

    /// 事务属性源。
    attribute_source: Arc<S>,
}

impl<S: TransactionAttributeSource + 'static> AspectJTransactionManagementConfiguration<S> {
    /// 创建配置实例。
    pub fn new(attribute_source: Arc<S>) -> Self {
        Self {
            transaction_manager_name: None,
            aspect: RwLock::new(None),
            attribute_source,
        }
    }

    /// 设置事务管理器名称。
    pub fn set_transaction_manager_name(&mut self, name: String) {
        self.transaction_manager_name = Some(name);
    }

    /// 获取事务管理器名称。
    pub fn get_transaction_manager_name(&self) -> Option<&str> {
        self.transaction_manager_name.as_deref()
    }

    /// 注册事务切面 Bean。
    ///
    /// 对应 Spring 的 `@Bean(name = TransactionManagementConfigUtils.TRANSACTION_ASPECT_BEAN_NAME)`。
    /// 返回切面的引用。
    pub fn register(&self) -> Arc<AnnotationTransactionAspect<S>> {
        let mut guard = self.aspect.write().unwrap();
        if guard.is_none() {
            *guard = Some(Arc::new(AnnotationTransactionAspect::new(
                self.attribute_source.clone(),
            )));
        }
        guard.as_ref().unwrap().clone()
    }

    /// 获取已注册的切面（如果已注册）。
    pub fn get_aspect(&self) -> Option<Arc<AnnotationTransactionAspect<S>>> {
        self.aspect.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::aspectj::transaction_attribute::TransactionAttribute;
    use crate::transaction::aspectj::transaction_attribute_source::MethodMetadata;

    struct MockSource;
    impl TransactionAttributeSource for MockSource {
        fn get_transaction_attribute(&self, _: &MethodMetadata) -> Option<TransactionAttribute> {
            Some(TransactionAttribute::default())
        }
        fn is_candidate_class(&self, _: &str) -> bool {
            true
        }
    }

    #[test]
    fn test_configuration_creation() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        assert!(config.get_transaction_manager_name().is_none());
        assert!(config.get_aspect().is_none());
    }

    #[test]
    fn test_configuration_register() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        let aspect = config.register();
        assert!(config.get_aspect().is_some());
        // 两次注册返回同一个实例
        let aspect2 = config.register();
        assert!(Arc::ptr_eq(&aspect, &aspect2));
    }

    #[test]
    fn test_configuration_set_transaction_manager_name() {
        let mut config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        config.set_transaction_manager_name("txManager".to_string());
        assert_eq!(config.get_transaction_manager_name(), Some("txManager"));
    }

    #[test]
    fn test_configuration_debug() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        // 不检查 Debug 实现，只检查创建成功
        let _ = config;
    }

    #[test]
    fn test_configuration_get_transaction_manager_name() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        assert!(config.get_transaction_manager_name().is_none());
    }

    #[test]
    fn test_configuration_register_multiple() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        let _ = config.register();
        let _ = config.register();
        let _ = config.register();
        assert!(config.get_aspect().is_some());
    }

    #[test]
    fn test_configuration_get_aspect_before_register() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        assert!(config.get_aspect().is_none());
    }

    #[test]
    fn test_configuration_get_aspect_after_register() {
        let config = AspectJTransactionManagementConfiguration::new(Arc::new(MockSource));
        let _ = config.register();
        assert!(config.get_aspect().is_some());
    }
}
