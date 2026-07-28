//! 对标 `org.springframework.transaction.annotation.Isolation` 枚举。
//!
//! 事务隔离级别定义了并发事务之间的数据可见性。

/// 事务隔离级别。
///
/// 对标 Spring 的 `Isolation` 枚举，定义 5 种隔离级别：
///
/// | 变体 | SQL 等价 | 脏读 | 不可重复读 | 幻读 |
/// |---|---|---|---|---|
/// | `Default` | 数据库默认 | 取决于数据库 | 取决于数据库 | 取决于数据库 |
/// | `ReadUncommitted` | READ UNCOMMITTED | 是 | 是 | 是 |
/// | `ReadCommitted` | READ COMMITTED | 否 | 是 | 是 |
/// | `RepeatableRead` | REPEATABLE READ | 否 | 否 | 是 |
/// | `Serializable` | SERIALIZABLE | 否 | 否 | 否 |
///
/// # Example
///
/// ```rust
/// use vernal_aspects::transaction::aspectj::Isolation;
///
/// assert_eq!(Isolation::default(), Isolation::Default);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Isolation {
    /// 使用数据库默认隔离级别（默认行为）。
    ///
    /// 对应 Spring 的 `Isolation.DEFAULT`。
    #[default]
    Default,

    /// 读未提交：最低隔离级别，可读到未提交数据。
    ///
    /// 对应 Spring 的 `Isolation.READ_UNCOMMITTED`。
    ReadUncommitted,

    /// 读已提交：只读已提交数据，不可重复读。
    ///
    /// 对应 Spring 的 `Isolation.READ_COMMITTED`。
    ReadCommitted,

    /// 可重复读：同一事务内多次读取结果一致。
    ///
    /// 对应 Spring 的 `Isolation.REPEATABLE_READ`。
    RepeatableRead,

    /// 串行化：最高隔离级别，完全串行执行。
    ///
    /// 对应 Spring 的 `Isolation.SERIALIZABLE`。
    Serializable,
}

impl Isolation {
    /// 返回隔离级别对应的超时秒数（0 表示使用数据库默认）。
    ///
    /// 用于与 JDBC 连接池配置时传递隔离级别参数。
    pub fn as_jdbc_value(self) -> i32 {
        match self {
            Self::Default => -1,
            Self::ReadUncommitted => 1,
            Self::ReadCommitted => 2,
            Self::RepeatableRead => 4,
            Self::Serializable => 8,
        }
    }

    /// 从 JDBC 隔离级别数值解析为 `Isolation`。
    ///
    /// 未知数值映射为 `Default`。
    pub fn from_jdbc_value(value: i32) -> Self {
        match value {
            1 => Self::ReadUncommitted,
            2 => Self::ReadCommitted,
            4 => Self::RepeatableRead,
            8 => Self::Serializable,
            _ => Self::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_default() {
        assert_eq!(Isolation::default(), Isolation::Default);
    }

    #[test]
    fn test_isolation_all_variants() {
        let variants = [
            Isolation::Default,
            Isolation::ReadUncommitted,
            Isolation::ReadCommitted,
            Isolation::RepeatableRead,
            Isolation::Serializable,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_isolation_jdbc_values() {
        assert_eq!(Isolation::Default.as_jdbc_value(), -1);
        assert_eq!(Isolation::ReadUncommitted.as_jdbc_value(), 1);
        assert_eq!(Isolation::ReadCommitted.as_jdbc_value(), 2);
        assert_eq!(Isolation::RepeatableRead.as_jdbc_value(), 4);
        assert_eq!(Isolation::Serializable.as_jdbc_value(), 8);
    }

    #[test]
    fn test_isolation_from_jdbc_value() {
        assert_eq!(Isolation::from_jdbc_value(-1), Isolation::Default);
        assert_eq!(Isolation::from_jdbc_value(1), Isolation::ReadUncommitted);
        assert_eq!(Isolation::from_jdbc_value(2), Isolation::ReadCommitted);
        assert_eq!(Isolation::from_jdbc_value(4), Isolation::RepeatableRead);
        assert_eq!(Isolation::from_jdbc_value(8), Isolation::Serializable);
        assert_eq!(Isolation::from_jdbc_value(999), Isolation::Default);
    }

    #[test]
    fn test_isolation_is_clone() {
        let i = Isolation::Serializable;
        let i2 = i;
        assert_eq!(i, i2);
    }

    #[test]
    fn test_isolation_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Isolation>();
        assert_sync::<Isolation>();
    }

    #[test]
    fn test_isolation_debug() {
        assert_eq!(format!("{:?}", Isolation::RepeatableRead), "RepeatableRead");
    }

    #[test]
    fn test_isolation_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Isolation::ReadCommitted, "rc");
        map.insert(Isolation::RepeatableRead, "rr");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_isolation_jdbc_roundtrip() {
        for isolation in [
            Isolation::Default,
            Isolation::ReadUncommitted,
            Isolation::ReadCommitted,
            Isolation::RepeatableRead,
            Isolation::Serializable,
        ] {
            let jdbc_val = isolation.as_jdbc_value();
            let roundtrip = Isolation::from_jdbc_value(jdbc_val);
            assert_eq!(isolation, roundtrip);
        }
    }

    #[test]
    fn test_isolation_all_variants_debug() {
        let variants = [
            Isolation::Default,
            Isolation::ReadUncommitted,
            Isolation::ReadCommitted,
            Isolation::RepeatableRead,
            Isolation::Serializable,
        ];
        for variant in variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_isolation_all_variants_clone() {
        let variants = [
            Isolation::Default,
            Isolation::ReadUncommitted,
            Isolation::ReadCommitted,
            Isolation::RepeatableRead,
            Isolation::Serializable,
        ];
        for variant in variants {
            let cloned = variant;
            assert_eq!(variant, cloned);
        }
    }

    #[test]
    fn test_isolation_all_variants_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Isolation::Default, 1);
        map.insert(Isolation::ReadUncommitted, 2);
        map.insert(Isolation::ReadCommitted, 3);
        map.insert(Isolation::RepeatableRead, 4);
        map.insert(Isolation::Serializable, 5);
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_isolation_from_jdbc_value_unknown() {
        assert_eq!(Isolation::from_jdbc_value(999), Isolation::Default);
    }

    #[test]
    fn test_isolation_as_jdbc_value() {
        assert_eq!(Isolation::Default.as_jdbc_value(), -1);
        assert_eq!(Isolation::ReadUncommitted.as_jdbc_value(), 1);
        assert_eq!(Isolation::ReadCommitted.as_jdbc_value(), 2);
        assert_eq!(Isolation::RepeatableRead.as_jdbc_value(), 4);
        assert_eq!(Isolation::Serializable.as_jdbc_value(), 8);
    }
}
