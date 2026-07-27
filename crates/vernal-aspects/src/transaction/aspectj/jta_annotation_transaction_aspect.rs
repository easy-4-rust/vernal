//! 对标 `org.springframework.transaction.aspectj.JtaAnnotationTransactionAspect`。
//!
//! 基于 JTA 1.2 `jakarta.transaction.Transactional` 注解驱动的事务切面。

use std::any::Any;
use std::sync::Arc;

use super::abstract_transaction_aspect::AbstractTransactionAspect;
use super::transaction_aspect_support::TransactionResult;
use super::transaction_attribute_source::{MethodMetadata, TransactionAttributeSource};

/// 基于 JTA 1.2 `@Transactional` 注解的事务切面。
///
/// 对标 Spring 的 `JtaAnnotationTransactionAspect`（`@RequiredTypes("jakarta.transaction.Transactional")`）。
/// 与 `AnnotationTransactionAspect` 类似，但使用 JTA 标准注解。
///
/// # 使用条件
///
/// 此切面需要 JTA 1.2（`jakarta.transaction`）在 classpath 上。
/// 通过 Cargo feature `jta` 启用。
pub struct JtaAnnotationTransactionAspect<S: TransactionAttributeSource> {
    /// 抽象事务切面基类。
    inner: AbstractTransactionAspect<S>,
}

impl<S: TransactionAttributeSource> JtaAnnotationTransactionAspect<S> {
    /// 创建 JTA 注解驱动的事务切面。
    ///
    /// 对应 Spring 的 `public JtaAnnotationTransactionAspect()` 构造函数。
    pub fn new(attribute_source: Arc<S>) -> Self {
        Self {
            inner: AbstractTransactionAspect::new(attribute_source),
        }
    }

    /// 匹配 JTA `@Transactional` 类型中的公开方法。
    ///
    /// 对应 Spring 的 pointcut：
    /// ```text
    /// execution(public * ((@Transactional *)+).*(..)) && within(@Transactional *)
    /// ```
    ///
    /// 注意：此处的 `@Transactional` 是 `jakarta.transaction.Transactional`。
    pub fn matches_jta_type_pointcut(
        &self,
        _type_has_jta_transactional: bool,
        _type_in_jta_transactional_scope: bool,
    ) -> bool {
        _type_has_jta_transactional && _type_in_jta_transactional_scope
    }

    /// 匹配 JTA `@Transactional` 方法。
    ///
    /// 对应 Spring 的 pointcut：
    /// ```text
    /// execution(@Transactional * *(..))
    /// ```
    pub fn matches_jta_method_pointcut(&self, _method_has_jta_transactional: bool) -> bool {
        _method_has_jta_transactional
    }

    /// 组合 pointcut。
    ///
    /// 对应 Spring 的 `transactionalMethodExecution(Object txObject)`。
    pub fn transactional_method_execution(
        &self,
        method: &MethodMetadata,
        type_has_jta: bool,
        type_in_scope: bool,
        method_has_jta: bool,
        this_matches: bool,
    ) -> bool {
        if !this_matches {
            return false;
        }
        self.matches_jta_type_pointcut(type_has_jta, type_in_scope)
            || self.matches_jta_method_pointcut(method_has_jta)
    }

    /// 在事务内执行方法。
    pub fn invoke_within_transaction<F>(
        &self,
        method: &MethodMetadata,
        target_type_name: &str,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        self.inner
            .invoke_within_transaction(method, target_type_name, callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::aspectj::transaction_attribute::TransactionAttribute;
    use std::sync::Arc;

    struct MockSource;
    impl TransactionAttributeSource for MockSource {
        fn get_transaction_attribute(&self, _: &MethodMetadata) -> Option<TransactionAttribute> {
            Some(TransactionAttribute::default())
        }
        fn is_candidate_class(&self, _: &str) -> bool { true }
    }

    #[test]
    fn test_jta_aspect_creation() {
        let aspect = JtaAnnotationTransactionAspect::new(Arc::new(MockSource));
        assert!(aspect.matches_jta_type_pointcut(true, true));
        assert!(!aspect.matches_jta_type_pointcut(false, true));
    }

    #[test]
    fn test_jta_method_pointcut() {
        let aspect = JtaAnnotationTransactionAspect::new(Arc::new(MockSource));
        assert!(aspect.matches_jta_method_pointcut(true));
        assert!(!aspect.matches_jta_method_pointcut(false));
    }

    #[test]
    fn test_jta_invoke_success() {
        let aspect = JtaAnnotationTransactionAspect::new(Arc::new(MockSource));
        let method = MethodMetadata::new("Foo", "bar", vec![], "void");
        let result = aspect.invoke_within_transaction(&method, "Foo", || {
            Ok(Box::new(1) as Box<dyn Any + Send + Sync>)
        });
        assert!(matches!(result, TransactionResult::Ok(_)));
    }
}
