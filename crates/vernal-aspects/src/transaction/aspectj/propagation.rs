//! 对标 `org.springframework.transaction.annotation.Propagation` 枚举。
//!
//! 事务传播行为定义了事务方法的调用方式：是否参与当前事务、创建新事务、或抛出异常。

/// 事务传播行为。
///
/// 对标 Spring 的 `Propagation` 枚举，定义 7 种事务传播行为：
///
/// | 变体 | 语义 |
/// |---|---|
/// | `Required` | 默认：支持当前事务，不存在则新建 |
/// | `Supports` | 支持当前事务，不存在则以非事务方式运行 |
/// | `Mandatory` | 必须在现有事务中运行，否则抛异常 |
/// | `RequiresNew` | 总是新建事务，挂起现有事务 |
/// | `NotSupported` | 以非事务方式运行，挂起现有事务 |
/// | `Never` | 不允许事务，存在则抛异常 |
/// | `Nested` | 嵌套事务（SavePoint） |
///
/// # Example
///
/// ```rust
/// use vernal_aspects::transaction::aspectj::Propagation;
///
/// assert_eq!(Propagation::default(), Propagation::Required);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Propagation {
    /// 支持当前事务，不存在则新建（默认行为）。
    ///
    /// 对应 Spring 的 `Propagation.REQUIRED`。
    #[default]
    Required,

    /// 支持当前事务，不存在则以非事务方式执行。
    ///
    /// 对应 Spring 的 `Propagation.SUPPORTS`。
    Supports,

    /// 必须在现有事务中运行，否则抛出 `TransactionException`。
    ///
    /// 对应 Spring 的 `Propagation.MANDATORY`。
    Mandatory,

    /// 总是创建新事务；如果存在当前事务则挂起。
    ///
    /// 对应 Spring 的 `Propagation.REQUIRES_NEW`。
    RequiresNew,

    /// 以非事务方式运行；如果存在当前事务则挂起。
    ///
    /// 对应 Spring 的 `Propagation.NOT_SUPPORTED`。
    NotSupported,

    /// 以非事务方式运行；如果存在当前事务则抛出异常。
    ///
    /// 对应 Spring 的 `Propagation.NEVER`。
    Never,

    /// 在嵌套事务中执行（使用 SavePoint）。
    ///
    /// 仅当存在事务时生效，内层回滚不影响外层。
    /// 对应 Spring 的 `Propagation.NESTED`。
    Nested,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_default_is_required() {
        assert_eq!(Propagation::default(), Propagation::Required);
    }

    #[test]
    fn test_propagation_all_variants() {
        let variants = [
            Propagation::Required,
            Propagation::Supports,
            Propagation::Mandatory,
            Propagation::RequiresNew,
            Propagation::NotSupported,
            Propagation::Never,
            Propagation::Nested,
        ];
        assert_eq!(variants.len(), 7);
    }

    #[test]
    fn test_propagation_is_clone() {
        let p = Propagation::RequiresNew;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn test_propagation_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Propagation>();
        assert_sync::<Propagation>();
    }

    #[test]
    fn test_propagation_debug() {
        let p = Propagation::Nested;
        assert_eq!(format!("{:?}", p), "Nested");
    }

    #[test]
    fn test_propagation_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Propagation::Required, "req");
        map.insert(Propagation::RequiresNew, "req_new");
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&Propagation::Required), Some(&"req"));
    }
}
