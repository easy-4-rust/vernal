//! 对标 `org.springframework.cache.aspectj.AnyThrow` 工具类。
//!
//! checked 异常透传辅助：在 AspectJ around advice 中将 checked 异常伪装成 unchecked 异常抛出。

/// checked 异常透传器。
///
/// 对标 Spring 的 `AnyThrow` 私有工具类。
///
/// # 用途
///
/// AspectJ 的 `around` advice 只能声明抛出 `RuntimeException` 和 `Error`，
/// 但 `CacheOperationInvoker#invoke()` 方法签名声明了 `throws Throwable`。
/// `AnyThrow` 利用 Java 泛型擦除机制，将 checked 异常伪装成 unchecked 异常抛出。
///
/// # Rust 替代
///
/// Rust 没有 checked exception 概念，所有异常都是 unchecked。
/// 此工具保留为 catch_unwind 集成测试的辅助层。
pub struct AnyThrow;

impl AnyThrow {
    /// 透传 checked 异常（作为 panic）。
    ///
    /// 对应 Spring 的 `AnyThrow.throwUnchecked(Throwable e)` 方法。
    pub fn throw_unchecked_panic(message: impl Into<String>) -> ! {
        panic!("{}", message.into());
    }

    /// 尝试执行闭包，如果 panic 则返回错误信息。
    ///
    /// 对应 Spring 的 try-catch-AnyThrow 模式。
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
    fn test_any_throw_try_execute_success() {
        let result = AnyThrow::try_execute(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_any_throw_try_execute_panic() {
        let result = AnyThrow::try_execute(|| {
            panic!("cache error");
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cache error"));
    }

    #[test]
    fn test_any_throw_try_execute_string_panic() {
        let result = AnyThrow::try_execute(|| {
            let msg = String::from("key not found");
            panic!("{}", msg);
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("key not found"));
    }

    #[test]
    fn test_any_throw_try_execute_unknown_panic() {
        let result = AnyThrow::try_execute(|| {
            panic!("numeric panic");
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("numeric panic"));
    }

    #[test]
    fn test_any_throw_try_execute_no_panic() {
        let result = AnyThrow::try_execute(|| "hello");
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_any_throw_try_execute_with_return_value() {
        let result = AnyThrow::try_execute(|| {
            let mut v = Vec::new();
            v.push(1);
            v.push(2);
            v
        });
        assert_eq!(result.unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_any_throw_try_execute_with_string_return() {
        let result = AnyThrow::try_execute(|| {
            String::from("test")
        });
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_any_throw_try_execute_with_option_return() {
        let result = AnyThrow::try_execute(|| {
            Some(42)
        });
        assert_eq!(result.unwrap(), Some(42));
    fn test_any_throw_try_execute_with_vec_return() {
        let result = AnyThrow::try_execute(|| {
            vec![1, 2, 3, 4, 5]
        });
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_any_throw_try_execute_with_map_return() {
        let result = AnyThrow::try_execute(|| {
            let mut map = std::collections::HashMap::new();
            map.insert("key1", "value1");
            map.insert("key2", "value2");
            map
        });
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_any_throw_try_execute_with_option_return() {
        let result = AnyThrow::try_execute(|| {
            Some(42)
        });
        assert_eq!(result.unwrap(), Some(42));
    }

    #[test]
    fn test_any_throw_try_execute_with_result_return() {
        let result = AnyThrow::try_execute(|| {
            Ok::<i32, String>(42)
        });
        assert_eq!(result.unwrap(), Ok(42));
    }
}
}
