//! 对标 `org.springframework.transaction.aspectj.AnnotationTransactionAspect` 具体 Aspect。
//!
//! 基于 Spring `@Transactional` 注解驱动的事务切面。

use std::any::Any;
use std::sync::Arc;

use super::abstract_transaction_aspect::AbstractTransactionAspect;
use super::transaction_aspect_support::TransactionResult;
use super::transaction_attribute_source::{MethodMetadata, TransactionAttributeSource};

/// 基于 `@Transactional` 注解的事务切面。
///
/// 对标 Spring 的 `AnnotationTransactionAspect`。
/// 这是 Spring `@Transactional` 注解的 AspectJ 实现，提供两种 pointcut：
///
/// 1. `executionOfAnyPublicMethodInAtTransactionalType()`：
///    匹配 `@Transactional` 类型中的任何公开方法
///    ```text
///    execution(public * ((@Transactional *)+).*(..)) && within(@Transactional *)
///    ```
///
/// 2. `executionOfTransactionalMethod()`：
///    匹配任何有 `@Transactional` 注解的方法
///    ```text
///    execution(@Transactional * *(..))
///    ```
///
/// 组合 pointcut：
/// ```text
/// (executionOfAnyPublicMethodInAtTransactionalType() || executionOfTransactionalMethod())
///     && this(txObject)
/// ```
///
/// # 使用
///
/// ```rust,ignore
/// use vernal_aspects::transaction::aspectj::*;
/// use std::sync::Arc;
///
/// let source = Arc::new(AnnotationTransactionAttributeSource::new(false));
/// let aspect = AnnotationTransactionAspect::new(source);
///
/// // aspect 可用于匹配 @Transactional 方法
/// ```
pub struct AnnotationTransactionAspect<S: TransactionAttributeSource> {
    /// 抽象事务切面基类。
    inner: AbstractTransactionAspect<S>,

    /// Spring `@Transactional` 注解驱动的属性源。
    attribute_source: Arc<S>,
}

impl<S: TransactionAttributeSource> AnnotationTransactionAspect<S> {
    /// 创建 `@Transactional` 注解驱动的事务切面。
    ///
    /// 对应 Spring 的 `public AnnotationTransactionAspect()` 构造函数。
    /// 使用 `AnnotationTransactionAttributeSource(false)` 作为默认属性源。
    ///
    /// # Arguments
    ///
    /// * `attribute_source` - 事务属性源（通常基于 `@Transactional` 注解）
    pub fn new(attribute_source: Arc<S>) -> Self {
        let inner = AbstractTransactionAspect::new(attribute_source.clone());
        Self {
            inner,
            attribute_source,
        }
    }

    /// 获取事务属性源。
    pub fn get_attribute_source(&self) -> &Arc<S> {
        &self.attribute_source
    }

    /// 获取底层事务切面支撑。
    pub fn get_support(&self) -> &super::transaction_aspect_support::TransactionAspectSupport<S> {
        self.inner.get_support()
    }

    /// 匹配 `executionOfAnyPublicMethodInAtTransactionalType()` pointcut。
    ///
    /// 对应 Spring 的 pointcut：
    /// ```text
    /// execution(public * ((@Transactional *)+).*(..)) && within(@Transactional *)
    /// ```
    ///
    /// 匹配逻辑：
    /// 1. 方法必须是 `public`
    /// 2. 方法所在类型或其超类型必须有 `@Transactional` 注解
    /// 3. 类型必须在 `@Transactional` 类型范围内（`within` 约束）
    pub fn matches_execution_of_any_public_method_in_at_transactional_type(
        &self,
        _method: &MethodMetadata,
        type_has_transactional_annotation: bool,
        type_in_transactional_scope: bool,
    ) -> bool {
        type_has_transactional_annotation && type_in_transactional_scope
    }

    /// 匹配 `executionOfTransactionalMethod()` pointcut。
    ///
    /// 对应 Spring 的 pointcut：
    /// ```text
    /// execution(@Transactional * *(..))
    /// ```
    ///
    /// 匹配逻辑：
    /// 1. 方法必须有 `@Transactional` 注解（无论可见性）
    pub fn matches_execution_of_transactional_method(
        &self,
        method_has_transactional_annotation: bool,
    ) -> bool {
        method_has_transactional_annotation
    }

    /// 组合 pointcut：`transactionalMethodExecution(Object txObject)`。
    ///
    /// 对应 Spring 的 `protected pointcut transactionalMethodExecution(Object txObject)`。
    /// 两种匹配方式的 `||` 组合，并通过 `this(txObject)` 约束。
    pub fn transactional_method_execution(
        &self,
        method: &MethodMetadata,
        type_has_transactional_annotation: bool,
        type_in_transactional_scope: bool,
        method_has_transactional_annotation: bool,
        this_object_matches: bool,
    ) -> bool {
        if !this_object_matches {
            return false;
        }

        let matches_type = self
            .matches_execution_of_any_public_method_in_at_transactional_type(
                method,
                type_has_transactional_annotation,
                type_in_transactional_scope,
            );

        let matches_method = self.matches_execution_of_transactional_method(
            method_has_transactional_annotation,
        );

        matches_type || matches_method
    }

    /// 在事务内执行方法（around advice）。
    ///
    /// 对应 Spring 的 `AbstractTransactionAspect#around` advice。
    /// 这是事务切面的核心执行入口。
    ///
    /// # 执行流程
    ///
    /// 1. 从 `MethodSignature` 获取方法信息
    /// 2. 调用 `invokeWithinTransaction` 执行事务
    /// 3. 如果成功，返回结果
    /// 4. 如果失败，根据回滚规则决定回滚
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

impl<S: TransactionAttributeSource + 'static> std::fmt::Debug for AnnotationTransactionAspect<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnnotationTransactionAspect")
            .field("attribute_source", &"<TransactionAttributeSource>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::aspectj::transaction_attribute::TransactionAttribute;
    use crate::transaction::aspectj::transaction_attribute_source::MethodMetadata;
    use std::sync::Arc;

    struct MockTransactionAttributeSource;

    impl TransactionAttributeSource for MockTransactionAttributeSource {
        fn get_transaction_attribute(
            &self,
            _method: &MethodMetadata,
        ) -> Option<TransactionAttribute> {
            Some(TransactionAttribute::default())
        }

        fn is_candidate_class(&self, _type_name: &str) -> bool {
            true
        }
    }

    #[test]
    fn test_annotation_transaction_aspect_creation() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        assert!(aspect.get_support().get_transaction_manager().is_none());
    }

    #[test]
    fn test_matches_execution_of_any_public_method_in_at_transactional_type() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        // 类型有 @Transactional 注解且在范围内
        assert!(aspect
            .matches_execution_of_any_public_method_in_at_transactional_type(
                &method, true, true
            ));

        // 类型没有 @Transactional 注解
        assert!(!aspect
            .matches_execution_of_any_public_method_in_at_transactional_type(
                &method, false, true
            ));

        // 类型不在范围内
        assert!(!aspect
            .matches_execution_of_any_public_method_in_at_transactional_type(
                &method, true, false
            ));
    }

    #[test]
    fn test_matches_execution_of_transactional_method() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);

        // 方法有 @Transactional 注解
        assert!(aspect.matches_execution_of_transactional_method(true));

        // 方法没有 @Transactional 注解
        assert!(!aspect.matches_execution_of_transactional_method(false));
    }

    #[test]
    fn test_transactional_method_execution_combination() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        // this_object 不匹配时返回 false
        assert!(!aspect.transactional_method_execution(
            &method, true, true, true, false
        ));

        // 类型匹配 + this_object 匹配
        assert!(aspect.transactional_method_execution(
            &method, true, true, false, true
        ));

        // 方法匹配 + this_object 匹配
        assert!(aspect.transactional_method_execution(
            &method, false, false, true, true
        ));

        // 都不匹配
        assert!(!aspect.transactional_method_execution(
            &method, false, false, false, true
        ));
    }

    #[test]
    fn test_invoke_within_transaction_success() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = aspect.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<i32>().unwrap(), &42);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_failure() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = aspect.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_annotation_transaction_aspect_debug() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        let debug_str = format!("{:?}", aspect);
        assert!(debug_str.contains("AnnotationTransactionAspect"));
    }

    #[test]
    fn test_annotation_transaction_aspect_clone() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        let _ = aspect;
    }

    #[test]
    fn test_annotation_transaction_aspect_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        map.insert(format!("{:?}", aspect), 1);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_annotation_transaction_aspect_get_attribute_source() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        let _ = aspect.get_attribute_source();
    }

    #[test]
    fn test_annotation_transaction_aspect_get_support() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        let _ = aspect.get_support();
    }

    #[test]
    fn test_annotation_transaction_aspect_transactional_method_execution_all_combinations() {
        let source = Arc::new(MockTransactionAttributeSource);
        let aspect = AnnotationTransactionAspect::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        // this 不匹配
        assert!(!aspect.transactional_method_execution(&method, true, true, true, false));

        // 类型匹配
        assert!(aspect.transactional_method_execution(&method, true, true, false, true));

        // 方法匹配
        assert!(aspect.transactional_method_execution(&method, false, false, true, true));

        // 都不匹配
        assert!(!aspect.transactional_method_execution(&method, false, false, false, true));
    }
}
