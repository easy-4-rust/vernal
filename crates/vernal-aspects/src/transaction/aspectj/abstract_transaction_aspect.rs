//! 对标 `org.springframework.transaction.aspectj.AbstractTransactionAspect` 抽象 Aspect。
//!
//! 事务切面抽象层：封装 `TransactionAspectSupport` 的 `invokeWithinTransaction` 行为，
//! 由子类实现 `transactionalMethodExecution` pointcut。

use std::any::Any;
use std::sync::Arc;

use super::transaction_aspect_support::{TransactionAspectSupport, TransactionResult};
use super::transaction_attribute_source::MethodMetadata;
use super::transaction_attribute_source::TransactionAttributeSource;

/// 事务切面抽象基类。
///
/// 对标 Spring 的 `AbstractTransactionAspect`。
/// 这是 Spring 事务切面的"心脏"，封装了 `TransactionAspectSupport` 的核心执行逻辑。
///
/// # 设计
///
/// `AbstractTransactionAspect` 是一个抽象 aspect（AspectJ 中的 `public abstract aspect`），
/// 它定义了：
/// 1. `around` advice：在方法执行前后进行事务管理
/// 2. `transactionalMethodExecution` 抽象 pointcut：由子类实现
///
/// 子类（如 `AnnotationTransactionAspect`）提供具体的 pointcut 定义。
///
/// # 生命周期
///
/// 1. 初始化：`new(attribute_source)` → 设置事务属性源
/// 2. 使用：`invoke_within_transaction(method, target, callback)` → 执行事务
/// 3. 销毁：`destroy()` → 清理事务管理器缓存
///
/// # Example
///
/// ```rust,ignore
/// use vernal_aspects::transaction::aspectj::*;
/// use std::sync::Arc;
///
/// let source = Arc::new(AnnotationTransactionAttributeSource::new(false));
/// let aspect = AbstractTransactionAspect::new(source);
///
/// // 使用 aspect 执行事务方法
/// ```
pub struct AbstractTransactionAspect<S: TransactionAttributeSource> {
    /// 事务切面支撑（核心执行引擎）。
    support: TransactionAspectSupport<S>,
}

impl<S: TransactionAttributeSource> AbstractTransactionAspect<S> {
    /// 创建事务切面抽象实例。
    ///
    /// 对应 Spring 的 `protected AbstractTransactionAspect(TransactionAttributeSource tas)`。
    ///
    /// # Arguments
    ///
    /// * `attribute_source` - 事务属性源
    pub fn new(attribute_source: Arc<S>) -> Self {
        Self {
            support: TransactionAspectSupport::new(attribute_source),
        }
    }

    /// 获取事务切面支撑实例。
    ///
    /// 用于访问底层的事务管理器配置。
    pub fn get_support(&self) -> &TransactionAspectSupport<S> {
        &self.support
    }

    /// 获取可变的事务切面支撑实例。
    ///
    /// 用于修改事务管理器配置。
    pub fn get_support_mut(&mut self) -> &mut TransactionAspectSupport<S> {
        &mut self.support
    }

    /// 在事务内执行方法（核心方法）。
    ///
    /// 对应 Spring 的 `AbstractTransactionAspect#around` advice 内部逻辑。
    /// 这是事务切面的核心执行入口，由 around advice 调用。
    ///
    /// # 执行流程
    ///
    /// 1. 从 `TransactionAspectSupport` 获取事务属性
    /// 2. 根据传播行为决定事务行为
    /// 3. 执行目标方法（通过 `callback`）
    /// 4. 根据结果决定提交或回滚
    ///
    /// # Arguments
    ///
    /// * `method` - 方法元数据
    /// * `target_type_name` - 目标类的完全限定名
    /// * `callback` - 目标方法的执行回调
    ///
    /// # Returns
    ///
    /// `TransactionResult` 包含目标方法的执行结果或异常信息。
    pub fn invoke_within_transaction<F>(
        &self,
        method: &MethodMetadata,
        target_type_name: &str,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        self.support
            .invoke_within_transaction(method, target_type_name, callback)
    }

    /// 清理事务管理器缓存。
    ///
    /// 对应 Spring 的 `destroy()` 回调。
    /// 在切面销毁时调用，清理事务管理器缓存。
    pub fn destroy(&self) {
        self.support.clear_transaction_manager_cache();
    }
}

impl<S: TransactionAttributeSource> std::fmt::Debug for AbstractTransactionAspect<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbstractTransactionAspect")
            .field("support", &"<TransactionAspectSupport>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::aspectj::transaction_attribute::TransactionAttribute;

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

    fn make_aspect() -> AbstractTransactionAspect<MockTransactionAttributeSource> {
        AbstractTransactionAspect::new(Arc::new(MockTransactionAttributeSource))
    }

    #[test]
    fn test_abstract_transaction_aspect_creation() {
        let aspect = make_aspect();
        assert!(aspect.get_support().get_transaction_manager().is_none());
    }

    #[test]
    fn test_destroy_does_not_panic() {
        let aspect = make_aspect();
        aspect.destroy();
    }

    #[test]
    fn test_invoke_within_transaction_callback_success() {
        let aspect = make_aspect();

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = aspect.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new("hello") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<&str>().unwrap(), &"hello");
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_callback_failure() {
        let aspect = make_aspect();

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
    fn test_get_support() {
        let aspect = make_aspect();
        let _ = aspect.get_support();
    }

    #[test]
    fn test_get_support_mut() {
        let mut aspect = make_aspect();
        let _ = aspect.get_support_mut();
    }

    #[test]
    fn test_abstract_transaction_aspect_debug() {
        let aspect = make_aspect();
        let debug_str = format!("{:?}", aspect);
        assert!(debug_str.contains("AbstractTransactionAspect"));
    }

    #[test]
    fn test_abstract_transaction_aspect_invoke_within_transaction_success() {
        let aspect = make_aspect();
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
    fn test_abstract_transaction_aspect_invoke_within_transaction_failure() {
        let aspect = make_aspect();
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
    fn test_abstract_transaction_aspect_get_support() {
        let aspect = make_aspect();
        let _ = aspect.get_support();
    }

    #[test]
    fn test_abstract_transaction_aspect_get_support_mut() {
        let mut aspect = make_aspect();
        let _ = aspect.get_support_mut();
    }

    #[test]
    fn test_abstract_transaction_aspect_destroy() {
        let aspect = make_aspect();
        aspect.destroy();
    }

    #[test]
    fn test_abstract_transaction_aspect_creation_with_support() {
        let aspect = make_aspect();
        let support = aspect.get_support();
        assert!(support.get_transaction_manager().is_none());
    }

    #[test]
    fn test_abstract_transaction_aspect_creation_with_support_mut() {
        let mut aspect = make_aspect();
        let support = aspect.get_support_mut();
        assert!(support.get_transaction_manager().is_none());
    }

    #[test]
    fn test_abstract_transaction_aspect_invoke_within_transaction_success_with_different_value() {
        let aspect = make_aspect();
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = aspect.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new("hello") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<&str>().unwrap(), &"hello");
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn test_abstract_transaction_aspect_invoke_within_transaction_failure_with_different_error() {
        let aspect = make_aspect();
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = aspect.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("different error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }
}
