//! 对标 `org.springframework.transaction.aspectj.AspectJJtaTransactionManagementConfiguration`。
//!
//! JTA 版 `@Configuration`：在 `AspectJTransactionManagementConfiguration` 基础上
//! 额外注册 `JtaAnnotationTransactionAspect`。

use std::sync::{Arc, RwLock};

use super::aspectj_transaction_management_configuration::AspectJTransactionManagementConfiguration;
use super::jta_annotation_transaction_aspect::JtaAnnotationTransactionAspect;
use super::transaction_attribute_source::TransactionAttributeSource;

/// AspectJ JTA 事务管理配置。
///
/// 对标 Spring 的 `AspectJJtaTransactionManagementConfiguration extends AspectJTransactionManagementConfiguration`。
/// 除了注册 `AnnotationTransactionAspect`（继承自父类），还额外注册 `JtaAnnotationTransactionAspect`。
pub struct AspectJJtaTransactionManagementConfiguration<S: TransactionAttributeSource + 'static> {
    /// 父类配置（Spring `@Configuration` 继承）。
    base: AspectJTransactionManagementConfiguration<S>,

    /// JTA 事务切面（懒初始化）。
    jta_aspect: RwLock<Option<Arc<JtaAnnotationTransactionAspect<S>>>>,
}

impl<S: TransactionAttributeSource + 'static> AspectJJtaTransactionManagementConfiguration<S> {
    /// 创建 JTA 配置实例。
    pub fn new(attribute_source: Arc<S>) -> Self {
        Self {
            base: AspectJTransactionManagementConfiguration::new(attribute_source),
            jta_aspect: RwLock::new(None),
        }
    }

    /// 获取基础配置。
    pub fn get_base(&self) -> &AspectJTransactionManagementConfiguration<S> {
        &self.base
    }

    /// 注册 JTA 事务切面 Bean。
    ///
    /// 对应 Spring 的 `@Bean(name = TransactionManagementConfigUtils.JTA_TRANSACTION_ASPECT_BEAN_NAME)`。
    pub fn register_jta_aspect(&self) -> Arc<JtaAnnotationTransactionAspect<S>> {
        let mut guard = self.jta_aspect.write().unwrap();
        if guard.is_none() {
            // 使用与基础配置相同的属性源
            let source = self.base.get_aspect()
                .map(|a| a.get_attribute_source().clone())
                .unwrap_or_else(|| Arc::new(panic!("Base aspect not registered")));
            *guard = Some(Arc::new(JtaAnnotationTransactionAspect::new(source)));
        }
        guard.as_ref().unwrap().clone()
    }

    /// 获取已注册的 JTA 切面。
    pub fn get_jta_aspect(&self) -> Option<Arc<JtaAnnotationTransactionAspect<S>>> {
        self.jta_aspect.read().unwrap().clone()
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
        fn is_candidate_class(&self, _: &str) -> bool { true }
    }

    #[test]
    fn test_jta_configuration_creation() {
        let config = AspectJJtaTransactionManagementConfiguration::new(Arc::new(MockSource));
        assert!(config.get_jta_aspect().is_none());
    }
}
