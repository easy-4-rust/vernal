//! 对标 `org.springframework.transaction.aspectj.AbstractTransactionAspect.Rethrower`。
//!
//! checked 异常透传辅助工具。在 AspectJ around advice 中，checked 异常
//! 不能直接声明抛出，需要通过类型擦除来透传。

/// checked 异常透传器。
///
/// 对标 Spring 的 `AbstractTransactionAspect.Rethrower` 私有静态类。
///
/// # 用途
///
/// AspectJ 的 `around` advice 只能声明抛出 `RuntimeException` 和 `Error`，
/// 但 Java 的 `invokeWithinTransaction` 方法签名声明了 `throws Throwable`。
/// `Rethrower` 利用 Java 泛型擦除机制，将 checked 异常伪装成 unchecked 异常抛出。
///
/// # Rust 替代
///
/// Rust 没有 checked exception 概念，所有异常都是 unchecked。
/// 此工具保留为 catch_unwind 集成测试的辅助层，用于模拟 Spring 的行为。
///
/// # Example
///
/// ```rust,ignore
/// use vernal_aspects::transaction::aspectj::rethrower::Rethrower;
///
/// // 在 catch_unwind 中模拟 checked 异常透传
/// let result = std::panic::catch_unwind(|| {
///     Rethrower::rethrow_panic("io error");
/// });
/// assert!(result.is_err());
/// ```
pub struct Rethrower;

impl Rethrower {
    /// 透传 checked 异常（作为 panic）。
    ///
    /// 对应 Spring 的 `Rethrower.rethrow(Throwable exception)` 方法。
    /// 在 Rust 中使用 `panic!` 模拟 checked 异常的透传。
    ///
    /// # Safety
    ///
    /// 此方法会 panic，应仅在 `catch_unwind` 上下文中调用。
    pub fn rethrow_panic(message: impl Into<String>) -> ! {
        panic!("{}", message.into());
    }

    /// 尝试执行闭包，如果 panic 则返回错误信息。
    ///
    /// 对应 Spring 的 try-catch-Rethrower 模式。
    pub fn try_execute<F, T>(f: F) -> Result<T, String>
    where
        F: FnOnce() -> T + std::panic::UnwindSafe,
    {
        match std::panic::catch_unwind(f) {
            Ok(val) => Ok(val),
            Err(panic) => {
                if let Some(msg) = panic.downcast_ref::<&str>() {
                    Err(msg.to_string())
                } else if let Some(msg) = panic.downcast_ref::<String>() {
                    Err(msg.clone())
                } else {
                    Err("Unknown panic".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rethrower_try_execute_success() {
        let result = Rethrower::try_execute(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_rethrower_try_execute_panic_str() {
        let result = Rethrower::try_execute(|| {
            panic!("io error");
        });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "io error");
    }

    #[test]
    fn test_rethrower_try_execute_panic_string() {
        let result = Rethrower::try_execute(|| {
            let msg = String::from("database error");
            panic!("{}", msg);
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("database error"));
    }

    #[test]
    fn test_rethrower_try_execute_panic_unknown() {
        let result = Rethrower::try_execute(|| {
            panic!("numeric panic code");
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("numeric panic code"));
    }

    #[test]
    fn test_rethrower_try_execute_no_panic() {
        let result = Rethrower::try_execute(|| "hello");
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_rethrower_try_execute_with_return_value() {
        let result = Rethrower::try_execute(|| {
            let mut v = Vec::new();
            v.push(1);
            v.push(2);
            v
        });
        assert_eq!(result.unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_rethrower_try_execute_with_option_return() {
        let result = Rethrower::try_execute(|| {
            Some(42)
        });
        assert_eq!(result.unwrap(), Some(42));
    }

    #[test]
    fn test_rethrower_try_execute_with_result_return() {
        let result = Rethrower::try_execute(|| {
            Ok::<i32, String>(42)
        });
        assert_eq!(result.unwrap(), Ok(42));
    }

    #[test]
    fn test_rethrower_try_execute_with_string_return() {
        let result = Rethrower::try_execute(|| {
            String::from("test")
        });
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_rethrower_try_execute_with_vec_return() {
        let result = Rethrower::try_execute(|| {
            vec![1, 2, 3, 4, 5]
        });
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_rethrower_try_execute_with_map_return() {
        let result = Rethrower::try_execute(|| {
            let mut map = std::collections::HashMap::new();
            map.insert("key1", "value1");
            map.insert("key2", "value2");
            map
        });
        assert_eq!(result.unwrap().len(), 2);
    }
}
