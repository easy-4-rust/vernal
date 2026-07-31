//! 对标 `org.springframework.transaction.interceptor.TransactionAttribute` 接口。
//!
//! 事务属性定义了一个事务方法的传播行为、隔离级别、回滚规则等元数据。

use std::borrow::Cow;

use super::isolation::Isolation;
use super::propagation::Propagation;

/// 默认超时秒数，表示使用数据库默认超时。
pub const TIMEOUT_DEFAULT: u64 = 0;

/// 事务属性。
///
/// 对标 Spring 的 `TransactionAttribute` 接口（实际上是对 `TransactionDefinition` 的扩展），
/// 描述一个事务方法的所有属性：传播行为、隔离级别、只读标志、超时、回滚规则等。
///
/// # 字段
///
/// | 字段 | Java 对应 | 说明 |
/// |---|---|---|
/// | `propagation` | `getPropagationBehavior()` | 传播行为 |
/// | `isolation` | `getIsolationLevel()` | 隔离级别 |
/// | `timeout` | `getTimeout()` | 超时秒数 |
/// | `read_only` | `isReadOnly()` | 是否只读 |
/// | `rollback_for` | `rollbackOn(Throwable)` | 需要回滚的异常类型名 |
/// | `no_rollback_for` | `noRollbackOn(Throwable)` | 不回滚的异常类型名 |
/// | `name` | `getName()` | 事务名称（可选）|
/// | `qualifier` | `getQualifier()` | 事务管理器限定名（可选）|
/// | `label` | `getLabels()` | 标签集合（5.3+ 新增）|
///
/// # Example
///
/// ```rust
/// use vernal_aspects::transaction::aspectj::{TransactionAttribute, Propagation, Isolation};
///
/// let attr = TransactionAttribute {
///     propagation: Propagation::RequiresNew,
///     isolation: Isolation::ReadCommitted,
///     read_only: true,
///     ..Default::default()
/// };
///
/// assert_eq!(attr.propagation, Propagation::RequiresNew);
/// assert!(attr.read_only);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAttribute {
    /// 事务传播行为。
    ///
    /// 对应 Spring 的 `Propagation propagationBehavior`。
    pub propagation: Propagation,

    /// 事务隔离级别。
    ///
    /// 对应 Spring 的 `Isolation isolationLevel`。
    pub isolation: Isolation,

    /// 是否只读事务。
    ///
    /// 对应 Spring 的 `boolean readOnly`。
    pub read_only: bool,

    /// 超时秒数（0 表示使用数据库默认）。
    ///
    /// 对应 Spring 的 `int timeout`（`TIMEOUT_DEFAULT = -1`）。
    pub timeout: u64,

    /// 需要回滚的异常类型名列表。
    ///
    /// 对应 Spring 的 `Set<Class<? extends Throwable>> rollbackOn`。
    pub rollback_for: Vec<Cow<'static, str>>,

    /// 不需要回滚的异常类型名列表。
    ///
    /// 对应 Spring 的 `Set<Class<? extends Throwable>> noRollbackOn`。
    pub no_rollback_for: Vec<Cow<'static, str>>,

    /// 事务名称（可选）。
    ///
    /// 对应 Spring 的 `String name`。
    pub name: Option<Cow<'static, str>>,

    /// 事务管理器限定名（可选）。
    ///
    /// 对应 Spring 的 `String qualifier`。
    pub qualifier: Option<Cow<'static, str>>,

    /// 标签集合（5.3+ 新增）。
    ///
    /// 对应 Spring 的 `Set<String> labels`。
    pub labels: Vec<Cow<'static, str>>,
}

impl Default for TransactionAttribute {
    fn default() -> Self {
        Self {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout: TIMEOUT_DEFAULT,
            rollback_for: Vec::new(),
            no_rollback_for: Vec::new(),
            name: None,
            qualifier: None,
            labels: Vec::new(),
        }
    }
}

impl TransactionAttribute {
    /// 判断给定的异常类型是否应该回滚。
    ///
    /// 对标 Spring 的 `TransactionAttribute#rollbackOn(Throwable)` 方法。
    ///
    /// # 回滚规则
    ///
    /// 1. 检查 `no_rollback_for` 列表：如果异常类名在列表中，不回滚
    /// 2. 检查 `rollback_for` 列表：如果列表非空，只在列表中的异常类型回滚
    /// 3. 默认规则：`RuntimeException` 和 `Error` 回滚，checked 异常不回滚
    ///
    /// # Arguments
    ///
    /// * `exception_type_name` - 异常类型的完全限定名
    /// * `is_runtime` - 是否是 `RuntimeException`（含子类）
    /// * `is_error` - 是否是 `Error`（含子类）
    ///
    /// # Example
    ///
    /// ```rust
    /// use vernal_aspects::transaction::aspectj::TransactionAttribute;
    ///
    /// let attr = TransactionAttribute::default();
    /// // RuntimeException 默认回滚
    /// assert!(attr.should_rollback("java.lang.IllegalArgumentException", true, false));
    /// // Error 默认回滚
    /// assert!(attr.should_rollback("java.lang.Error", false, true));
    /// // checked 异常默认不回滚
    /// assert!(!attr.should_rollback("java.io.IOException", false, false));
    /// ```
    pub fn should_rollback(
        &self,
        exception_type_name: &str,
        is_runtime: bool,
        is_error: bool,
    ) -> bool {
        // 检查 no_rollback_for 列表
        for no_rb in &self.no_rollback_for {
            if exception_type_name == no_rb.as_ref() {
                return false;
            }
        }

        // 检查 rollback_for 列表（非空时只有列表中的异常才回滚）
        if !self.rollback_for.is_empty() {
            for rb in &self.rollback_for {
                if exception_type_name == rb.as_ref() {
                    return true;
                }
            }
            return false;
        }

        // 默认规则：RuntimeException 和 Error 回滚
        is_runtime || is_error
    }

    /// 返回事务管理器的限定名（Bean 名或 qualifier）。
    ///
    /// 对应 Spring 的 `TransactionAttribute#getQualifier()`。
    pub fn get_qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }

    /// 返回事务名称。
    ///
    /// 对应 Spring 的 `TransactionAttribute#getName()`。
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 返回标签集合。
    ///
    /// 对应 Spring 的 `TransactionAttribute#getLabels()`。
    pub fn get_labels(&self) -> &[Cow<'static, str>] {
        &self.labels
    }

    /// 设置事务管理器限定名。
    ///
    /// 对应 Spring 的 `TransactionAttribute#setQualifier(String)`。
    pub fn set_qualifier(&mut self, qualifier: impl Into<Cow<'static, str>>) {
        self.qualifier = Some(qualifier.into());
    }

    /// 设置事务名称。
    ///
    /// 对应 Spring 的 `TransactionAttribute#setName(String)`。
    pub fn set_name(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = Some(name.into());
    }

    /// 设置标签集合。
    ///
    /// 对应 Spring 的 `TransactionAttribute#setLabels(Set<String>)`。
    pub fn set_labels(&mut self, labels: Vec<Cow<'static, str>>) {
        self.labels = labels;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_transaction_attribute() {
        let attr = TransactionAttribute::default();
        assert_eq!(attr.propagation, Propagation::Required);
        assert_eq!(attr.isolation, Isolation::Default);
        assert!(!attr.read_only);
        assert_eq!(attr.timeout, TIMEOUT_DEFAULT);
        assert!(attr.rollback_for.is_empty());
        assert!(attr.no_rollback_for.is_empty());
        assert!(attr.name.is_none());
        assert!(attr.qualifier.is_none());
        assert!(attr.labels.is_empty());
    }

    #[test]
    fn test_should_rollback_default_runtime_exception() {
        let attr = TransactionAttribute::default();
        // RuntimeException 默认回滚
        assert!(attr.should_rollback("java.lang.IllegalArgumentException", true, false));
        assert!(attr.should_rollback("java.lang.NullPointerException", true, false));
    }

    #[test]
    fn test_should_rollback_default_error() {
        let attr = TransactionAttribute::default();
        // Error 默认回滚
        assert!(attr.should_rollback("java.lang.Error", false, true));
        assert!(attr.should_rollback("java.lang.OutOfMemoryError", false, true));
    }

    #[test]
    fn test_should_rollback_default_checked_exception() {
        let attr = TransactionAttribute::default();
        // checked 异常默认不回滚
        assert!(!attr.should_rollback("java.io.IOException", false, false));
        assert!(!attr.should_rollback("java.sql.SQLException", false, false));
    }

    #[test]
    fn test_should_rollback_with_rollback_for() {
        let mut attr = TransactionAttribute::default();
        attr.rollback_for
            .push(Cow::Borrowed("java.io.IOException"));

        // 在列表中的 checked 异常回滚
        assert!(attr.should_rollback("java.io.IOException", false, false));

        // 不在列表中的 checked 异常不回滚
        assert!(!attr.should_rollback("java.sql.SQLException", false, false));

        // 列表非空时，RuntimeException 也不再默认回滚（只有列表中的才回滚）
        // 注意：这是 Spring 的行为——当 rollbackFor 非空时，只回滚列表中的异常
        assert!(!attr.should_rollback("java.lang.IllegalArgumentException", true, false));

        // Error 也不再默认回滚（当 rollbackFor 非空时）
        assert!(!attr.should_rollback("java.lang.Error", false, true));
    }

    #[test]
    fn test_should_rollback_with_no_rollback_for() {
        let mut attr = TransactionAttribute::default();
        attr.no_rollback_for
            .push(Cow::Borrowed("java.lang.IllegalArgumentException"));

        // 在 no_rollback_for 列表中的异常不回滚
        assert!(!attr.should_rollback("java.lang.IllegalArgumentException", true, false));

        // 不在列表中的 RuntimeException 仍然回滚
        assert!(attr.should_rollback("java.lang.NullPointerException", true, false));

        // checked 异常仍然不回滚
        assert!(!attr.should_rollback("java.io.IOException", false, false));
    }

    #[test]
    fn test_should_rollback_combined_rules() {
        let mut attr = TransactionAttribute::default();
        attr.rollback_for
            .push(Cow::Borrowed("java.io.IOException"));
        attr.no_rollback_for
            .push(Cow::Borrowed("java.io.FileNotFoundException"));

        // FileNotFoundException 在 no_rollback_for 列表中，不回滚
        assert!(!attr.should_rollback("java.io.FileNotFoundException", false, false));

        // IOException 在 rollback_for 列表中且不在 no_rollback_for 中，回滚
        assert!(attr.should_rollback("java.io.IOException", false, false));

        // 其他 checked 异常不回滚
        assert!(!attr.should_rollback("java.sql.SQLException", false, false));
    }

    #[test]
    fn test_getter_setter() {
        let mut attr = TransactionAttribute::default();

        attr.set_name("testTx");
        assert_eq!(attr.get_name(), Some("testTx"));

        attr.set_qualifier("txManager");
        assert_eq!(attr.get_qualifier(), Some("txManager"));

        attr.set_labels(vec![Cow::Borrowed("audit"), Cow::Borrowed("log")]);
        assert_eq!(attr.get_labels().len(), 2);
    }

    #[test]
    fn test_custom_transaction_attribute() {
        let attr = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            isolation: Isolation::ReadCommitted,
            read_only: true,
            timeout: 30,
            ..Default::default()
        };

        assert_eq!(attr.propagation, Propagation::RequiresNew);
        assert_eq!(attr.isolation, Isolation::ReadCommitted);
        assert!(attr.read_only);
        assert_eq!(attr.timeout, 30);
    }

    #[test]
    fn test_transaction_attribute_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TransactionAttribute>();
        assert_sync::<TransactionAttribute>();
    }

    #[test]
    fn test_transaction_attribute_clone() {
        let attr = TransactionAttribute {
            propagation: Propagation::Nested,
            ..Default::default()
        };
        let cloned = attr.clone();
        assert_eq!(attr, cloned);
    }

    #[test]
    fn test_should_rollback_empty_rollback_for() {
        let mut attr = TransactionAttribute::default();
        attr.rollback_for.push(Cow::Borrowed("java.io.IOException"));

        // 列表非空时，只有列表中的异常才回滚
        assert!(attr.should_rollback("java.io.IOException", false, false));
        // RuntimeException 不再默认回滚（当 rollbackFor 非空时）
        assert!(!attr.should_rollback("java.lang.IllegalArgumentException", true, false));
        // Error 不再默认回滚（当 rollbackFor 非空时）
        assert!(!attr.should_rollback("java.lang.Error", false, true));
    }

    #[test]
    fn test_should_rollback_no_rollback_for_runtime() {
        let mut attr = TransactionAttribute::default();
        attr.no_rollback_for
            .push(Cow::Borrowed("java.lang.IllegalArgumentException"));

        // 在 no_rollback_for 列表中的 RuntimeException 不回滚
        assert!(!attr.should_rollback("java.lang.IllegalArgumentException", true, false));
        // 不在列表中的 RuntimeException 回滚
        assert!(attr.should_rollback("java.lang.NullPointerException", true, false));
    }

    #[test]
    fn test_should_rollback_empty_lists() {
        let attr = TransactionAttribute::default();
        // 空列表：RuntimeException 和 Error 回滚，checked 异常不回滚
        assert!(attr.should_rollback("java.lang.IllegalArgumentException", true, false));
        assert!(attr.should_rollback("java.lang.Error", false, true));
        assert!(!attr.should_rollback("java.io.IOException", false, false));
    }

    #[test]
    fn test_should_rollback_all_runtime_exceptions() {
        let mut attr = TransactionAttribute::default();
        attr.no_rollback_for
            .push(Cow::Borrowed("java.lang.NullPointerException"));

        // NullPointerException 不回滚
        assert!(!attr.should_rollback("java.lang.NullPointerException", true, false));
        // 其他 RuntimeException 回滚
        assert!(attr.should_rollback("java.lang.IllegalArgumentException", true, false));
        assert!(attr.should_rollback("java.lang.IllegalStateException", true, false));
    }

    #[test]
    fn test_should_rollback_all_errors() {
        let mut attr = TransactionAttribute::default();
        attr.no_rollback_for
            .push(Cow::Borrowed("java.lang.OutOfMemoryError"));

        // OutOfMemoryError 不回滚
        assert!(!attr.should_rollback("java.lang.OutOfMemoryError", false, true));
        // 其他 Error 回滚
        assert!(attr.should_rollback("java.lang.StackOverflowError", false, true));
        assert!(attr.should_rollback("java.lang.Error", false, true));
    }

    #[test]
    fn test_should_rollback_all_checked_exceptions() {
        let mut attr = TransactionAttribute::default();
        attr.rollback_for
            .push(Cow::Borrowed("java.io.IOException"));

        // IOException 回滚
        assert!(attr.should_rollback("java.io.IOException", false, false));
        // 其他 checked 异常不回滚
        assert!(!attr.should_rollback("java.sql.SQLException", false, false));
    }

    #[test]
    fn test_transaction_attribute_default_all_fields() {
        let attr = TransactionAttribute::default();
        assert_eq!(attr.propagation, Propagation::Required);
        assert_eq!(attr.isolation, Isolation::Default);
        assert!(!attr.read_only);
        assert_eq!(attr.timeout, 0);
        assert!(attr.rollback_for.is_empty());
        assert!(attr.no_rollback_for.is_empty());
        assert!(attr.name.is_none());
        assert!(attr.qualifier.is_none());
        assert!(attr.labels.is_empty());
    }

    #[test]
    fn test_transaction_attribute_full_construction() {
        let attr = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            isolation: Isolation::Serializable,
            read_only: true,
            timeout: 60,
            rollback_for: vec![Cow::Borrowed("java.io.IOException")],
            no_rollback_for: vec![Cow::Borrowed("java.io.FileNotFoundException")],
            name: Some(Cow::Borrowed("testTx")),
            qualifier: Some(Cow::Borrowed("txManager")),
            labels: vec![Cow::Borrowed("audit"), Cow::Borrowed("log")],
        };

        assert_eq!(attr.propagation, Propagation::RequiresNew);
        assert_eq!(attr.isolation, Isolation::Serializable);
        assert!(attr.read_only);
        assert_eq!(attr.timeout, 60);
        assert_eq!(attr.rollback_for.len(), 1);
        assert_eq!(attr.no_rollback_for.len(), 1);
        assert_eq!(attr.name.as_deref(), Some("testTx"));
        assert_eq!(attr.qualifier.as_deref(), Some("txManager"));
        assert_eq!(attr.labels.len(), 2);
    }

    #[test]
    fn test_transaction_attribute_debug() {
        let attr = TransactionAttribute::default();
        let debug_str = format!("{:?}", attr);
        assert!(debug_str.contains("Required"));
    }

    #[test]
    fn test_transaction_attribute_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let attr1 = TransactionAttribute::default();
        let attr2 = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            ..Default::default()
        };
        map.insert(format!("{:?}", attr1), 1);
        map.insert(format!("{:?}", attr2), 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_transaction_attribute_with_all_fields() {
        let attr = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            isolation: Isolation::Serializable,
            read_only: true,
            timeout: 60,
            rollback_for: vec![Cow::Borrowed("java.io.IOException")],
            no_rollback_for: vec![Cow::Borrowed("java.io.FileNotFoundException")],
            name: Some(Cow::Borrowed("testTx")),
            qualifier: Some(Cow::Borrowed("txManager")),
            labels: vec![Cow::Borrowed("audit"), Cow::Borrowed("log")],
        };

        assert_eq!(attr.propagation, Propagation::RequiresNew);
        assert_eq!(attr.isolation, Isolation::Serializable);
        assert!(attr.read_only);
        assert_eq!(attr.timeout, 60);
        assert_eq!(attr.rollback_for.len(), 1);
        assert_eq!(attr.no_rollback_for.len(), 1);
        assert_eq!(attr.name.as_deref(), Some("testTx"));
        assert_eq!(attr.qualifier.as_deref(), Some("txManager"));
        assert_eq!(attr.labels.len(), 2);
    }

    #[test]
    fn test_transaction_attribute_with_no_labels() {
        let attr = TransactionAttribute {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout: 0,
            rollback_for: Vec::new(),
            no_rollback_for: Vec::new(),
            name: None,
            qualifier: None,
            labels: Vec::new(),
        };

        assert_eq!(attr.propagation, Propagation::Required);
        assert_eq!(attr.isolation, Isolation::Default);
        assert!(!attr.read_only);
        assert_eq!(attr.timeout, 0);
        assert!(attr.rollback_for.is_empty());
        assert!(attr.no_rollback_for.is_empty());
        assert!(attr.name.is_none());
        assert!(attr.qualifier.is_none());
        assert!(attr.labels.is_empty());
    }

    #[test]
    fn test_should_rollback_with_multiple_rollback_for() {
        let mut attr = TransactionAttribute::default();
        attr.rollback_for.push(Cow::Borrowed("java.io.IOException"));
        attr.rollback_for.push(Cow::Borrowed("java.sql.SQLException"));

        assert!(attr.should_rollback("java.io.IOException", false, false));
        assert!(attr.should_rollback("java.sql.SQLException", false, false));
        assert!(!attr.should_rollback("java.lang.RuntimeException", true, false));
    }

    #[test]
    fn test_should_rollback_with_multiple_no_rollback_for() {
        let mut attr = TransactionAttribute::default();
        attr.no_rollback_for.push(Cow::Borrowed("java.lang.NullPointerException"));
        attr.no_rollback_for.push(Cow::Borrowed("java.lang.IllegalStateException"));

        assert!(!attr.should_rollback("java.lang.NullPointerException", true, false));
        assert!(!attr.should_rollback("java.lang.IllegalStateException", true, false));
        assert!(attr.should_rollback("java.lang.RuntimeException", true, false));
    }

    #[test]
    fn test_transaction_attribute_debug_with_labels() {
        let attr = TransactionAttribute {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout: 0,
            rollback_for: Vec::new(),
            no_rollback_for: Vec::new(),
            name: Some(Cow::Borrowed("test")),
            qualifier: None,
            labels: vec![Cow::Borrowed("audit")],
        };
        let debug_str = format!("{:?}", attr);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("audit"));
    }
}
