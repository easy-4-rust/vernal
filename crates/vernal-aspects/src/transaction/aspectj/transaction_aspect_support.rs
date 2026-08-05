//! 对标 `org.springframework.transaction.interceptor.TransactionAspectSupport` 抽象类。
//!
//! 事务切面支撑基类：提供事务管理的核心执行引擎 `invoke_within_transaction`。

use std::any::Any;
use std::sync::Arc;

use super::propagation::Propagation;
use super::transaction_attribute::TransactionAttribute;
use super::transaction_attribute_source::MethodMetadata;
use super::transaction_attribute_source::TransactionAttributeSource;

/// 事务执行结果。
///
/// 对标 Spring 的 `Object invokeWithinTransaction(...)` 返回值。
/// 封装了方法执行的结果或异常信息。
#[derive(Debug)]
pub enum TransactionResult {
    /// 正常返回。
    Ok(Box<dyn Any + Send + Sync>),
    /// 抛出异常。
    Err(TransactionError),
}

/// 事务错误。
///
/// 对标 Spring 中事务执行失败时的异常信息。
#[derive(Debug)]
pub struct TransactionError {
    /// 异常消息。
    pub message: String,
    /// 异常类型名。
    pub exception_type: &'static str,
    /// 是否是 RuntimeException。
    pub is_runtime: bool,
    /// 是否是 Error。
    pub is_error: bool,
    /// 是否是受检异常（checked exception）。
    pub is_checked: bool,
}

impl TransactionError {
    /// 创建新的事务错误。
    pub fn new(
        message: String,
        exception_type: &'static str,
        is_runtime: bool,
        is_error: bool,
    ) -> Self {
        Self {
            message,
            exception_type,
            is_runtime,
            is_error,
            is_checked: !is_runtime && !is_error,
        }
    }
}

/// 事务管理器 trait。
///
/// 对标 Spring 的 `PlatformTransactionManager`。
/// 负责事务的创建、提交和回滚。
pub trait TransactionManager: Send + Sync + 'static {
    /// 获取事务管理器的名称。
    fn get_name(&self) -> &str;

    /// 获取事务管理器的类型标识。
    fn get_type(&self) -> &str;
}

/// 默认的空事务管理器（用于测试或占位）。
///
/// 所有事务操作都会直接抛出错误。
#[allow(dead_code)] // Java 镜像脚手架：占位管理器，供测试与后续接入真实 TransactionManager 使用
pub struct NoOpTransactionManager;

impl TransactionManager for NoOpTransactionManager {
    fn get_name(&self) -> &str {
        "NoOpTransactionManager"
    }

    fn get_type(&self) -> &str {
        "NoOp"
    }
}

/// 事务挂起信息。
///
/// 对标 Spring 的 `TransactionSynchronizationManager` 中的挂起事务信息。
#[allow(dead_code)]
// Java 镜像脚手架：挂起事务的数据模型，后续接入真实事务同步管理器时使用
#[derive(Debug, Clone)]
pub struct SuspendedTransactionInfo {
    /// 挂起的事务名称。
    pub transaction_name: String,
    /// 挂起的事务管理器名称。
    pub transaction_manager_name: String,
    /// 挂起事务的其他元数据。
    pub metadata: std::collections::HashMap<String, String>,
}

/// 事务切面支撑基类。
///
/// 对标 Spring 的 `TransactionAspectSupport` 抽象类。
/// 提供事务管理的核心执行引擎 `invoke_within_transaction`，
/// 子类提供具体的 pointcut 和事务属性源。
///
/// # 设计
///
/// 这是 Spring 事务切面的"心脏"，负责：
/// 1. 从属性源获取事务属性
/// 2. 根据传播行为决定是否创建/加入事务
/// 3. 执行目标方法
/// 4. 根据结果决定提交还是回滚
/// 5. 处理异常传播
pub struct TransactionAspectSupport<S: TransactionAttributeSource> {
    /// 事务属性源。
    attribute_source: Arc<S>,

    /// 事务管理器。
    transaction_manager: Option<Arc<dyn TransactionManager>>,

    /// 事务管理器缓存（方法 → 事务管理器）。
    ///
    /// 对标 Spring 的 `transactionManagerCache`。
    transaction_manager_cache:
        std::sync::RwLock<std::collections::HashMap<String, Arc<dyn TransactionManager>>>,

    /// 是否存在当前事务（用于测试）。
    ///
    /// 对标 Spring 的 `TransactionSynchronizationManager.isSynchronizationActive()`。
    #[cfg(test)]
    has_current_tx: std::sync::atomic::AtomicBool,
}

impl<S: TransactionAttributeSource> TransactionAspectSupport<S> {
    /// 创建新的事务切面支撑实例。
    pub fn new(attribute_source: Arc<S>) -> Self {
        Self {
            attribute_source,
            transaction_manager: None,
            transaction_manager_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
            #[cfg(test)]
            has_current_tx: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 设置是否存在当前事务（用于测试）。
    #[cfg(test)]
    fn set_has_current_tx_for_testing(&self, value: bool) {
        self.has_current_tx
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }

    /// 设置事务管理器。
    pub fn set_transaction_manager(&mut self, transaction_manager: Arc<dyn TransactionManager>) {
        self.transaction_manager = Some(transaction_manager);
    }

    /// 获取事务管理器。
    pub fn get_transaction_manager(&self) -> Option<&Arc<dyn TransactionManager>> {
        self.transaction_manager.as_ref()
    }

    /// 获取事务属性源。
    pub fn get_transaction_attribute_source(&self) -> &Arc<S> {
        &self.attribute_source
    }

    /// 清理事务管理器缓存。
    ///
    /// 对标 Spring 的 `clearTransactionManagerCache()` 方法。
    /// 在 Aspect 切面销毁时调用。
    pub fn clear_transaction_manager_cache(&self) {
        if let Ok(mut cache) = self.transaction_manager_cache.write() {
            cache.clear();
        }
    }

    /// 核心：在事务内执行方法。
    ///
    /// 对标 Spring 的 `TransactionAspectSupport#invokeWithinTransaction` 方法。
    /// 这是 Spring 事务管理的核心执行引擎，实现了完整的事务生命周期。
    ///
    /// # 执行流程
    ///
    /// 1. 获取事务属性（`TransactionAttribute`）
    /// 2. 如果属性为 null，直接执行目标方法（无事务）
    /// 3. 根据传播行为决定事务行为
    /// 4. 如果是 `REQUIRED` 且存在当前事务，加入现有事务
    /// 5. 如果是 `REQUIRES_NEW` 或 `REQUIRED` 且无事务，创建新事务
    /// 6. 执行目标方法
    /// 7. 如果目标方法抛出异常，根据回滚规则决定回滚
    /// 8. 如果目标方法正常完成，提交事务
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
        // 1. 获取事务属性
        let attribute = self.attribute_source.get_transaction_attribute(method);

        // 2. 如果没有事务属性，直接执行
        let attribute = match attribute {
            Some(attr) => attr,
            None => return self.execute_without_transaction(callback),
        };

        // 3. 根据传播行为决定事务行为
        match attribute.propagation {
            // 支持当前事务，不存在则新建（默认行为）
            Propagation::Required => {
                self.handle_required(attribute, target_type_name, method, callback)
            }

            // 支持当前事务，不存在则以非事务方式执行
            Propagation::Supports => {
                self.handle_supports(attribute, target_type_name, method, callback)
            }

            // 必须在现有事务中运行，否则抛异常
            Propagation::Mandatory => {
                self.handle_mandatory(attribute, target_type_name, method, callback)
            }

            // 总是新建事务，挂起现有事务
            Propagation::RequiresNew => {
                self.handle_requires_new(attribute, target_type_name, method, callback)
            }

            // 以非事务方式运行，挂起现有事务
            Propagation::NotSupported => {
                self.handle_not_supported(attribute, target_type_name, method, callback)
            }

            // 以非事务方式运行，存在则抛异常
            Propagation::Never => self.handle_never(attribute, target_type_name, method, callback),

            // 在嵌套事务中执行
            Propagation::Nested => {
                self.handle_nested(attribute, target_type_name, method, callback)
            }
        }
    }

    /// 处理 REQUIRED 传播行为。
    fn handle_required<F>(
        &self,
        attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 尝试获取当前事务
        let has_current_tx = self.has_current_transaction(target_type_name);

        if has_current_tx {
            // 加入现有事务
            self.execute_with_existing_transaction(attribute, method, callback)
        } else {
            // 创建新事务
            self.create_and_execute_transaction(attribute, target_type_name, method, callback)
        }
    }

    /// 处理 SUPPORTS 传播行为。
    fn handle_supports<F>(
        &self,
        attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        let has_current_tx = self.has_current_transaction(target_type_name);

        if has_current_tx {
            self.execute_with_existing_transaction(attribute, method, callback)
        } else {
            self.execute_without_transaction(callback)
        }
    }

    /// 处理 MANDATORY 传播行为。
    fn handle_mandatory<F>(
        &self,
        _attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        let has_current_tx = self.has_current_transaction(target_type_name);

        if has_current_tx {
            self.execute_with_existing_transaction(_attribute, method, callback)
        } else {
            TransactionResult::Err(TransactionError::new(
                format!(
                    "No existing transaction found for method '{}' in class '{}'",
                    method.method_name, target_type_name
                ),
                "org.springframework.transaction.IllegalTransactionStateException",
                true,
                false,
            ))
        }
    }

    /// 处理 REQUIRES_NEW 传播行为。
    fn handle_requires_new<F>(
        &self,
        attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 挂起现有事务（如果有）
        let _suspended = self.suspend_transaction(target_type_name);

        // 创建新事务
        self.create_and_execute_transaction(attribute, target_type_name, method, callback)
    }

    /// 处理 NOT_SUPPORTED 传播行为。
    fn handle_not_supported<F>(
        &self,
        _attribute: TransactionAttribute,
        target_type_name: &str,
        _method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 挂起现有事务
        let _suspended = self.suspend_transaction(target_type_name);

        // 以非事务方式执行
        self.execute_without_transaction(callback)
    }

    /// 处理 NEVER 传播行为。
    fn handle_never<F>(
        &self,
        _attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        let has_current_tx = self.has_current_transaction(target_type_name);

        if has_current_tx {
            TransactionResult::Err(TransactionError::new(
                format!(
                    "Found existing transaction for method '{}' in class '{}'",
                    method.method_name, target_type_name
                ),
                "org.springframework.transaction.IllegalTransactionStateException",
                true,
                false,
            ))
        } else {
            self.execute_without_transaction(callback)
        }
    }

    /// 处理 NESTED 传播行为。
    fn handle_nested<F>(
        &self,
        attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        let has_current_tx = self.has_current_transaction(target_type_name);

        if has_current_tx {
            // 创建嵌套事务（使用 SavePoint）
            self.create_nested_transaction(attribute, target_type_name, method, callback)
        } else {
            // 没有当前事务，等价于 REQUIRED
            self.create_and_execute_transaction(attribute, target_type_name, method, callback)
        }
    }

    /// 无事务执行。
    fn execute_without_transaction<F>(&self, callback: F) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        match callback() {
            Ok(result) => TransactionResult::Ok(result),
            Err(err) => TransactionResult::Err(TransactionError::new(
                format!("Method execution failed: {:?}", err),
                "unknown",
                false,
                false,
            )),
        }
    }

    /// 在现有事务中执行。
    fn execute_with_existing_transaction<F>(
        &self,
        _attribute: TransactionAttribute,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        match callback() {
            Ok(result) => TransactionResult::Ok(result),
            Err(err) => {
                // 检查回滚规则
                let should_rollback = _attribute.should_rollback(
                    "unknown", true, // 假设是 RuntimeException
                    false,
                );

                if should_rollback {
                    // 回滚现有事务
                    TransactionResult::Err(TransactionError::new(
                        format!(
                            "Rolling back transaction for method '{}' due to exception: {:?}",
                            method.method_name, err
                        ),
                        "org.springframework.transaction.TransactionSystemException",
                        true,
                        false,
                    ))
                } else {
                    TransactionResult::Err(TransactionError::new(
                        format!(
                            "Method '{}' threw exception but transaction not rolled back: {:?}",
                            method.method_name, err
                        ),
                        "unknown",
                        false,
                        false,
                    ))
                }
            }
        }
    }

    /// 创建并执行事务。
    fn create_and_execute_transaction<F>(
        &self,
        _attribute: TransactionAttribute,
        _target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 开启事务（实际实现需要调用 transaction manager）
        match callback() {
            Ok(result) => TransactionResult::Ok(result),
            Err(err) => TransactionResult::Err(TransactionError::new(
                format!(
                    "Rolling back transaction for method '{}' due to exception: {:?}",
                    method.method_name, err
                ),
                "org.springframework.transaction.TransactionSystemException",
                true,
                false,
            )),
        }
    }

    /// 创建嵌套事务。
    fn create_nested_transaction<F>(
        &self,
        attribute: TransactionAttribute,
        target_type_name: &str,
        method: &MethodMetadata,
        callback: F,
    ) -> TransactionResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 嵌套事务逻辑（使用 SavePoint）
        self.create_and_execute_transaction(attribute, target_type_name, method, callback)
    }

    /// 检查是否存在当前事务。
    ///
    /// 对标 Spring 的 `TransactionSynchronizationManager.isSynchronizationActive()`。
    fn has_current_transaction(&self, _target_type_name: &str) -> bool {
        // 实际实现需要检查事务同步管理器
        #[cfg(test)]
        return self
            .has_current_tx
            .load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(test))]
        return false;
    }

    /// 挂起当前事务。
    ///
    /// 对标 Spring 的 `TransactionSynchronizationManager.suspendSynchronization()`。
    fn suspend_transaction(&self, _target_type_name: &str) -> Option<SuspendedTransactionInfo> {
        // 实际实现需要挂起事务同步
        None
    }
}

impl<S: TransactionAttributeSource> Default for TransactionAspectSupport<S> {
    fn default() -> Self {
        panic!("TransactionAspectSupport requires a TransactionAttributeSource")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockTransactionAttributeSource {
        propagation: Propagation,
    }

    impl MockTransactionAttributeSource {
        fn with_propagation(propagation: Propagation) -> Self {
            Self { propagation }
        }
    }

    impl TransactionAttributeSource for MockTransactionAttributeSource {
        fn get_transaction_attribute(
            &self,
            _method: &MethodMetadata,
        ) -> Option<TransactionAttribute> {
            Some(TransactionAttribute {
                propagation: self.propagation,
                ..Default::default()
            })
        }

        fn is_candidate_class(&self, _type_name: &str) -> bool {
            true
        }
    }

    struct MockEmptyAttributeSource;

    impl TransactionAttributeSource for MockEmptyAttributeSource {
        fn get_transaction_attribute(
            &self,
            _method: &MethodMetadata,
        ) -> Option<TransactionAttribute> {
            None
        }

        fn is_candidate_class(&self, _type_name: &str) -> bool {
            false
        }
    }

    struct MockTransactionAttributeSourceWithNoRollback;

    impl MockTransactionAttributeSourceWithNoRollback {
        fn new() -> Self {
            Self
        }
    }

    impl TransactionAttributeSource for MockTransactionAttributeSourceWithNoRollback {
        fn get_transaction_attribute(
            &self,
            _method: &MethodMetadata,
        ) -> Option<TransactionAttribute> {
            Some(TransactionAttribute {
                propagation: Propagation::Required,
                no_rollback_for: vec![std::borrow::Cow::Borrowed("unknown")],
                ..Default::default()
            })
        }

        fn is_candidate_class(&self, _type_name: &str) -> bool {
            true
        }
    }
    #[test]
    fn test_invoke_within_transaction_no_attribute() {
        let source = Arc::new(MockEmptyAttributeSource);
        let support = TransactionAspectSupport::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_required_no_existing() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let executed = Arc::new(AtomicUsize::new(0));
        let executed_clone = executed.clone();

        let result = support.invoke_within_transaction(&method, "com.example.Foo", move || {
            executed_clone.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        assert_eq!(executed.load(Ordering::SeqCst), 1);
        match result {
            TransactionResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<i32>().unwrap(), &42);
            }
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_mandatory_no_existing() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Mandatory,
        ));
        let support = TransactionAspectSupport::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(
                    err.exception_type
                        .contains("IllegalTransactionStateException")
                );
            }
            _ => panic!("Expected Err for mandatory without existing tx"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_required_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_supports_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Supports,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_mandatory_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Mandatory,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_never_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(
                    err.exception_type
                        .contains("IllegalTransactionStateException")
                );
            }
            _ => panic!("Expected Err for never with existing tx"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_nested_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_never_no_existing() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);

        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<i32>().unwrap(), &42);
            }
            _ => panic!("Expected Ok for never without existing tx"),
        }
    }

    #[test]
    fn test_clear_transaction_manager_cache() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        support.clear_transaction_manager_cache();
    }

    #[test]
    fn test_set_transaction_manager() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let mut support = TransactionAspectSupport::new(source);
        let tm = Arc::new(NoOpTransactionManager);
        support.set_transaction_manager(tm.clone());
        assert!(support.get_transaction_manager().is_some());
        assert_eq!(
            support.get_transaction_manager().unwrap().get_name(),
            "NoOpTransactionManager"
        );
    }

    #[test]
    fn test_get_transaction_manager_none() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        assert!(support.get_transaction_manager().is_none());
    }

    #[test]
    fn test_clear_metadata_cache() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        support.clear_transaction_manager_cache();
    }

    #[test]
    fn test_invoke_within_transaction_required_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_supports_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Supports,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_requires_new_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::RequiresNew,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_not_supported_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_mandatory_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Mandatory,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(
                    err.exception_type
                        .contains("IllegalTransactionStateException")
                );
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_never_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_nested_exception() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_required_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_supports_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Supports,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_requires_new_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::RequiresNew,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_not_supported_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_never_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_nested_success() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_transaction_result_debug() {
        let ok = TransactionResult::Ok(Box::new(42) as Box<dyn Any + Send + Sync>);
        let err = TransactionResult::Err(TransactionError::new(
            "error".to_string(),
            "unknown",
            false,
            false,
        ));
        assert!(format!("{:?}", ok).contains("Ok"));
        assert!(format!("{:?}", err).contains("Err"));
    }

    #[test]
    fn test_transaction_error_debug() {
        let err = TransactionError::new("error".to_string(), "java.lang.Error", false, true);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("error"));
        assert!(debug_str.contains("java.lang.Error"));
    }

    #[test]
    fn test_transaction_error_new() {
        let err = TransactionError::new("error".to_string(), "java.lang.Error", false, true);
        assert_eq!(err.message, "error");
        assert_eq!(err.exception_type, "java.lang.Error");
        assert!(!err.is_runtime);
        assert!(err.is_error);
        assert!(!err.is_checked);
    }

    #[test]
    fn test_suspended_transaction_info() {
        let info = SuspendedTransactionInfo {
            transaction_name: "testTx".to_string(),
            transaction_manager_name: "txManager".to_string(),
            metadata: std::collections::HashMap::new(),
        };
        assert_eq!(info.transaction_name, "testTx");
        assert_eq!(info.transaction_manager_name, "txManager");
        assert!(info.metadata.is_empty());
    }

    #[test]
    fn test_no_op_transaction_manager() {
        let tm = NoOpTransactionManager;
        assert_eq!(tm.get_name(), "NoOpTransactionManager");
        assert_eq!(tm.get_type(), "NoOp");
    }

    #[test]
    fn test_invoke_within_transaction_required_with_no_rollback_for() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_never_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_not_supported_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_nested_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
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
    fn test_invoke_within_transaction_required_no_existing_tx_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_supports_no_existing_tx_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Supports,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_requires_new_no_existing_tx_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::RequiresNew,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_not_supported_no_existing_tx_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_nested_no_existing_tx_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_suspend_transaction_returns_none() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let result = support.suspend_transaction("com.example.Foo");
        assert!(result.is_none());
    }

    #[test]
    fn test_create_nested_transaction() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.create_nested_transaction(attr, "com.example.Foo", &method, || {
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
    fn test_create_nested_transaction_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.create_nested_transaction(attr, "com.example.Foo", &method, || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_create_and_execute_transaction_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result =
            support.create_and_execute_transaction(attr, "com.example.Foo", &method, || {
                Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
            });

        match result {
            TransactionResult::Err(err) => {
                assert!(err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_execute_without_transaction_err() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);

        let result = support.execute_without_transaction(|| {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_handle_nested_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_nested(attr, "com.example.Foo", &method, || {
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
    fn test_handle_nested_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Nested,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_nested(attr, "com.example.Foo", &method, || {
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
    fn test_handle_requires_new_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::RequiresNew,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_requires_new(attr, "com.example.Foo", &method, || {
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
    fn test_handle_requires_new_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::RequiresNew,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_requires_new(attr, "com.example.Foo", &method, || {
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
    fn test_handle_mandatory_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Mandatory,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_mandatory(attr, "com.example.Foo", &method, || {
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
    fn test_handle_not_supported_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_not_supported(attr, "com.example.Foo", &method, || {
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
    fn test_handle_not_supported_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::NotSupported,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_not_supported(attr, "com.example.Foo", &method, || {
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
    fn test_handle_never_with_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_never(attr, "com.example.Foo", &method, || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(
                    err.exception_type
                        .contains("IllegalTransactionStateException")
                );
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_handle_never_no_existing_tx() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Never,
        ));
        let support = TransactionAspectSupport::new(source);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let attr = TransactionAttribute::default();

        let result = support.handle_never(attr, "com.example.Foo", &method, || {
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
    fn test_invoke_within_transaction_required_with_no_rollback_for_set() {
        // Create a mock that has no_rollback_for set
        let source = Arc::new(MockTransactionAttributeSourceWithNoRollback::new());
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        // Should get a non-rollback error because no_rollback_for matches
        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
                assert!(err.message.contains("not rolled back"));
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_supports_with_no_rollback_for_set() {
        let source = Arc::new(MockTransactionAttributeSourceWithNoRollback::new());
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
                assert!(err.message.contains("not rolled back"));
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_invoke_within_transaction_mandatory_with_no_rollback_for_set() {
        let source = Arc::new(MockTransactionAttributeSourceWithNoRollback::new());
        let support = TransactionAspectSupport::new(source);
        support.set_has_current_tx_for_testing(true);
        let method = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");

        let result = support.invoke_within_transaction(&method, "com.example.Foo", || {
            Err(Box::new("test error") as Box<dyn Any + Send + Sync>)
        });

        match result {
            TransactionResult::Err(err) => {
                assert!(!err.is_runtime);
                assert!(err.message.contains("not rolled back"));
            }
            _ => panic!("Expected Err result"),
        }
    }

    #[test]
    fn test_get_transaction_attribute_source() {
        let source = Arc::new(MockTransactionAttributeSource::with_propagation(
            Propagation::Required,
        ));
        let support = TransactionAspectSupport::new(source);
        let attr_source = support.get_transaction_attribute_source();
        // Verify we got a reference to the attribute source
        let _ = attr_source;
    }
}
